mod files;
mod multipart;
mod naming;
mod persist;
mod types;
mod validate;

pub use multipart::parse_upload_multipart;
pub use naming::{extension_of, new_job_id, sanitize_filename, slugify};
pub use types::{StagedFile, UploadFile, UploadPayload};
pub use validate::{needs_conversion, validate_extensions};

use leptos::prelude::ServerFnError;

use crate::app::model::MediaType;
use crate::app::server::AppState;
use crate::app::server::convert::{JobPhase, job_set_phase};

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
    Ok(format!("تم رفع {} ملف بنجاح", file_rows.len()))
}
