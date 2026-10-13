use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use server_fn::codec::{MultipartData, MultipartFormData};

use crate::app::error::AppError;

#[cfg(feature = "ssr")]
use crate::app::constants::messages;
#[cfg(feature = "ssr")]
use crate::app::error::ErrorContext;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadResult {
    pub success: bool,
    pub message: String,
    pub job_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConversionStatus {
    Writing,
    Converting {
        conversion_index: usize,
        conversion_count: usize,
        current_file: String,
        progress: f32,
    },
    Finalizing,
    Done,
    Failed(String),
}

#[server(input = MultipartFormData)]
pub async fn upload_media(data: MultipartData) -> Result<UploadResult, AppError> {
    use crate::app::model::MediaKind;
    use crate::app::server::convert::{Job, JobPhase};
    use crate::app::server::upload::{
        needs_conversion, parse_upload_multipart, process_upload, schedule_job_eviction,
        validate_extensions,
    };
    use crate::app::server::{AppState, db};

    let mut state: AppState = expect_context();
    let payload = parse_upload_multipart(data).await?;

    let kind_str = db::fetch_section_kind(&mut state.db, &payload.section_slug)
        .await
        .context("fetch section kind for upload")?
        .ok_or_else(|| AppError::public(messages::SECTION_NOT_FOUND))?;
    let kind = MediaKind::try_from(kind_str.as_str())
        .map_err(|e| AppError::internal("section has invalid media_kind", e))?;

    validate_extensions(&payload, kind)?;

    if !needs_conversion(&payload, kind) {
        let msg = process_upload(payload, &state, kind, None).await?;
        return Ok(UploadResult {
            success: true,
            message: msg,
            job_id: None,
        });
    }

    let job_id = uuid::Uuid::new_v4().to_string();
    {
        let mut jobs = state.jobs.write().await;
        jobs.insert(
            job_id.clone(),
            Job {
                phase: JobPhase::Writing,
                started_at: std::time::Instant::now(),
            },
        );
    }

    let state_bg = state.clone();
    let job_id_bg = job_id.clone();

    tokio::spawn(async move {
        if let Err(e) = process_upload(payload, &state_bg, kind, Some(&job_id_bg)).await {
            crate::app::server::convert::job_set_phase(
                &state_bg.jobs,
                Some(&job_id_bg),
                JobPhase::Failed(e.message.clone()),
            )
            .await;
            schedule_job_eviction(state_bg.jobs.clone(), job_id_bg);
        }
    });

    Ok(UploadResult {
        success: true,
        message: messages::UPLOAD_STARTED_BG.into(),
        job_id: Some(job_id),
    })
}

#[server]
pub async fn poll_conversion(job_id: String) -> Result<ConversionStatus, AppError> {
    use crate::app::server::AppState;
    use crate::app::server::convert::JobPhase;

    let state: AppState = expect_context();
    let jobs = state.jobs.read().await;
    let job = jobs
        .get(&job_id)
        .ok_or_else(|| AppError::public(messages::JOB_NOT_FOUND))?;

    Ok(match &job.phase {
        JobPhase::Writing => ConversionStatus::Writing,
        JobPhase::Converting {
            conversion_index,
            conversion_count,
            current_file,
            progress,
        } => ConversionStatus::Converting {
            conversion_index: *conversion_index,
            conversion_count: *conversion_count,
            current_file: current_file.clone(),
            progress: *progress,
        },
        JobPhase::Finalizing => ConversionStatus::Finalizing,
        JobPhase::Done => ConversionStatus::Done,
        JobPhase::Failed(e) => ConversionStatus::Failed(e.clone()),
    })
}
