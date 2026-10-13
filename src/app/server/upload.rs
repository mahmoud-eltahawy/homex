mod files;
mod multipart;
mod naming;
mod persist;
mod types;
mod validate;

use std::time::Duration;

use tokio_util::sync::CancellationToken;

pub use multipart::parse_upload_multipart;
pub use naming::{extension_of, sanitize_filename, slugify};
use tokio::time::sleep;
pub use types::{StagedFile, UploadFile, UploadPayload};
pub use validate::{needs_conversion, validate_extensions};

use crate::app::error::AppError;
use crate::app::model::MediaKind;
use crate::app::server::convert::{JobPhase, job_set_phase};
use crate::app::server::{AppState, Jobs};

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
    let mut db = state.db.clone();
    let file_rows = persist::insert_files(&mut db, staged).await?;
    persist::insert_items_and_poster(&mut db, payload, &file_rows, &state.config.storage.data_dir)
        .await
}

async fn finish_job(state: &AppState, job_id: Option<&str>) {
    job_set_phase(&state.jobs, job_id, JobPhase::Done).await;
    if let Some(id) = job_id {
        schedule_job_eviction(state.jobs.clone(), id.to_string());
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
    persist_phase(state, payload, &staged, job_id).await?;
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
