mod files;
mod incoming;
mod multipart;
mod naming;
mod persist;
mod return_to_incoming;
mod types;
mod validate;

use std::path::PathBuf;
use std::time::Duration;

use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub use incoming::{
    RawFile, collect_files, dominant_kind, ensure_kind_matches, read_immediate_subdirs,
};
pub use multipart::parse_upload_multipart;
pub use naming::{extension_of, sanitize_filename, sanitize_rel, slugify};
pub use return_to_incoming::move_file_to_incoming;
use tokio::time::sleep;
pub use types::{StagedFile, UploadFile, UploadPayload, UploadTarget};
pub use validate::{needs_conversion, validate_extensions};

use crate::app::error::AppError;
use crate::app::ids::Slug;
use crate::app::model::MediaKind;
use crate::app::server::convert::{JobPhase, job_set_phase};
use crate::app::server::poster::{extract_and_save_poster, pick_timestamp, write_poster};
use crate::app::server::{AppState, Jobs, db};

const JOB_RETENTION: Duration = Duration::from_secs(60);

pub fn schedule_job_eviction(jobs: Jobs, job_id: String) {
    tokio::spawn(async move {
        sleep(JOB_RETENTION).await;
        jobs.write().await.remove(&job_id);
    });
}

async fn stage_phase(
    payload: &UploadPayload,
    state: &AppState,
    kind: MediaKind,
    job_id: Option<&str>,
    cancel: &CancellationToken,
) -> Result<Vec<StagedFile>, AppError> {
    job_set_phase(&state.jobs, job_id, JobPhase::Writing).await;
    files::stage_files(payload, state, kind, job_id, cancel).await
}

async fn persist_phase(
    state: &AppState,
    payload: &UploadPayload,
    staged: &[StagedFile],
    job_id: Option<&str>,
) -> Result<(), AppError> {
    job_set_phase(&state.jobs, job_id, JobPhase::Finalizing).await;
    let file_rows = persist::insert_files(&state.db, staged).await?;
    persist::insert_items_and_poster(&state.db, payload, &file_rows).await
}

async fn finish_job(state: &AppState, job_id: Option<&str>) {
    job_set_phase(&state.jobs, job_id, JobPhase::Done).await;
    if let Some(id) = job_id {
        schedule_job_eviction(state.jobs.clone(), id.to_string());
    }
}

async fn extract_and_insert_new_collection(
    state: &AppState,
    payload: &UploadPayload,
    kind: MediaKind,
    staged: &[StagedFile],
) -> Result<Option<PathBuf>, AppError> {
    let UploadTarget::New {
        id,
        title,
        description,
        tags,
    } = &payload.target
    else {
        return Ok(None);
    };
    let id = *id;

    let db = state.db.clone();

    match kind {
        MediaKind::Video => {
            let first = staged
                .first()
                .ok_or_else(|| AppError::internal("extract poster", "no staged files"))?;
            let abs = state.config.storage.media_root.join(&first.rel);
            let duration = crate::app::server::convert::ffprobe_duration(&abs).await;
            let ts = pick_timestamp(duration, true);

            let url = extract_and_save_poster(
                &state.config.storage.data_dir,
                payload.section_slug.as_str(),
                id,
                &abs,
                ts,
            )
            .await?;

            let file_rel = url
                .strip_prefix(crate::app::constants::routes::POSTERS_URL_PREFIX)
                .map(|r| {
                    state
                        .config
                        .storage
                        .data_dir
                        .join(crate::app::constants::storage::POSTERS_DIR)
                        .join(r)
                });

            db::insert_video_collection(
                &db,
                id,
                &payload.section_slug,
                title,
                description.as_deref(),
                tags,
                &url,
            )
            .await
            .map_err(|e| AppError::internal("insert video collection", e))?;

            Ok(file_rel)
        }
        MediaKind::Audio => {
            let url = match payload.poster.as_ref() {
                Some((ext, bytes)) => Some(
                    write_poster(
                        &state.config.storage.data_dir,
                        payload.section_slug.as_str(),
                        id,
                        ext,
                        bytes,
                    )
                    .await?,
                ),
                None => None,
            };
            let file_rel = url.as_ref().and_then(|u| {
                u.strip_prefix(crate::app::constants::routes::POSTERS_URL_PREFIX)
                    .map(|r| {
                        state
                            .config
                            .storage
                            .data_dir
                            .join(crate::app::constants::storage::POSTERS_DIR)
                            .join(r)
                    })
            });

            db::insert_audio_collection(
                &db,
                id,
                &payload.section_slug,
                title,
                description.as_deref(),
                tags,
                url,
            )
            .await
            .map_err(|e| AppError::internal("insert audio collection", e))?;

            Ok(file_rel)
        }
    }
}

async fn execute_upload(
    payload: &UploadPayload,
    state: &AppState,
    kind: MediaKind,
    job_id: Option<&str>,
    cancel: &CancellationToken,
) -> Result<String, AppError> {
    let staged = stage_phase(payload, state, kind, job_id, cancel).await?;

    let is_new = matches!(payload.target, UploadTarget::New { .. });
    let mut poster_path: Option<PathBuf> = None;
    if is_new {
        poster_path = extract_and_insert_new_collection(state, payload, kind, &staged).await?;
    }

    let persist_result = persist_phase(state, payload, &staged, job_id).await;
    if let Err(e) = persist_result {
        if let Some(path) = poster_path {
            let _ = tokio::fs::remove_file(&path).await;
        }
        for sf in &staged {
            let staged_abs = state.config.storage.media_root.join(&sf.rel);
            if staged_abs.exists()
                && tokio::fs::rename(&staged_abs, &sf.origin).await.is_err()
                && tokio::fs::copy(&staged_abs, &sf.origin).await.is_ok()
            {
                let _ = tokio::fs::remove_file(&staged_abs).await;
            }
        }
        return Err(e);
    }

    finish_job(state, job_id).await;
    Ok(format!("Successfully uploaded {} file(s)", staged.len()))
}

pub async fn process_upload(
    payload: UploadPayload,
    state: &AppState,
    kind: MediaKind,
    job_id: Option<&str>,
    cancel: &CancellationToken,
) -> Result<String, AppError> {
    execute_upload(&payload, state, kind, job_id, cancel).await
}

pub async fn extract_poster_for_new_video_collection(
    state: &AppState,
    section_slug: &Slug,
    collection_id: Uuid,
    source_path: &std::path::Path,
    first_only: bool,
) -> Result<String, AppError> {
    let duration = crate::app::server::convert::ffprobe_duration(source_path).await;
    let ts = pick_timestamp(duration, first_only);
    extract_and_save_poster(
        &state.config.storage.data_dir,
        section_slug.as_str(),
        collection_id,
        source_path,
        ts,
    )
    .await
}
