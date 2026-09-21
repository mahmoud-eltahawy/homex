mod files;
mod multipart;
mod naming;
mod persist;
mod types;
mod validate;

use std::time::Duration;

pub use multipart::parse_upload_multipart;
pub use naming::{extension_of, new_job_id, new_storage_token, sanitize_filename, slugify};
use tokio::time::sleep;
pub use types::{StagedFile, UploadFile, UploadPayload};
pub use validate::{needs_conversion, validate_extensions};

use leptos::prelude::ServerFnError;

use crate::app::model::MediaKind;
use crate::app::server::convert::{JobPhase, job_set_phase};
use crate::app::server::{AppState, Jobs, SqlErr};

const JOB_RETENTION: Duration = Duration::from_secs(60);

pub fn schedule_job_eviction(jobs: Jobs, job_id: String) {
    tokio::spawn(async move {
        sleep(JOB_RETENTION).await;
        jobs.write().await.remove(&job_id);
    });
}

pub async fn process_upload(
    payload: UploadPayload,
    state: &AppState,
    kind: MediaKind,
    job_id: Option<&str>,
) -> Result<String, ServerFnError> {
    let collection_id = payload.collection_id;

    job_set_phase(&state.jobs, job_id, JobPhase::Writing).await;
    let staged = files::stage_files(&payload, state, kind, job_id).await?;

    job_set_phase(&state.jobs, job_id, JobPhase::Finalizing).await;
    let file_rows = persist::insert_files(&state.db, &staged).await?;

    let mut tx = state.db.begin().await.srv()?;
    persist::insert_items(&mut tx, collection_id, payload.season_number, &file_rows).await?;
    persist::attach_poster(
        &mut tx,
        &payload,
        collection_id,
        &state.config.storage.data_dir,
    )
    .await?;
    tx.commit().await.srv()?;

    job_set_phase(&state.jobs, job_id, JobPhase::Done).await;
    if let Some(id) = job_id {
        schedule_job_eviction(state.jobs.clone(), id.to_string());
    }
    Ok(format!("تم رفع {} ملف بنجاح", file_rows.len()))
}
