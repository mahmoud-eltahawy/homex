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

use crate::app::model::MediaType;
use crate::app::server::convert::{JobPhase, job_set_phase};
use crate::app::server::{AppState, Jobs};

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
    job_id: Option<&str>,
) -> Result<String, ServerFnError> {
    job_set_phase(&state.jobs, job_id, JobPhase::Writing).await;
    let staged = files::stage_files(&payload, state, job_id).await?;

    job_set_phase(&state.jobs, job_id, JobPhase::Finalizing).await;
    let file_rows = persist::insert_files(&state.db, &staged).await?;

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let entity_id = match payload.media_type {
        MediaType::Movie => persist::persist_movie(&mut tx, &payload, &file_rows).await?,
        MediaType::Series => persist::persist_series(&mut tx, &payload, &file_rows).await?,
        MediaType::AudioGroup => {
            persist::persist_audio_group(&mut tx, &payload, &file_rows).await?
        }
    };

    persist::attach_poster(&mut tx, &payload, entity_id, &state.config.storage.data_dir).await?;

    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    job_set_phase(&state.jobs, job_id, JobPhase::Done).await;
    if let Some(id) = job_id {
        schedule_job_eviction(state.jobs.clone(), id.to_string());
    }
    Ok(format!("تم رفع {} ملف بنجاح", file_rows.len()))
}
