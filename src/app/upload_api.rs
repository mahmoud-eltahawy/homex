use leptos::prelude::*;
use server_fn::codec::{MultipartData, MultipartFormData};

use crate::app::upload_ui::{ConversionStatus, UploadResult};

#[server(input = MultipartFormData)]
pub async fn upload_media(data: MultipartData) -> Result<UploadResult, ServerFnError> {
    use crate::app::server::AppState;
    use crate::app::server::convert::{Job, JobPhase};
    use crate::app::server::upload::{
        needs_conversion, new_job_id, parse_upload_multipart, process_upload,
        schedule_job_eviction, validate_extensions,
    };

    let state: AppState = expect_context();
    let payload = parse_upload_multipart(data).await?;
    validate_extensions(&payload)?;

    if !needs_conversion(&payload) {
        let msg = process_upload(payload, &state, None).await?;
        return Ok(UploadResult {
            success: true,
            message: msg,
            job_id: None,
        });
    }

    let job_id = new_job_id();
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
        if let Err(e) = process_upload(payload, &state_bg, Some(&job_id_bg)).await {
            crate::app::server::convert::job_set_phase(
                &state_bg.jobs,
                Some(&job_id_bg),
                JobPhase::Failed(e.to_string()),
            )
            .await;
            schedule_job_eviction(state_bg.jobs.clone(), job_id_bg);
        }
    });

    Ok(UploadResult {
        success: true,
        message: "بدأ رفع الملفات وتحويلها في الخلفية".into(),
        job_id: Some(job_id),
    })
}

#[server]
pub async fn poll_conversion(job_id: String) -> Result<ConversionStatus, ServerFnError> {
    use crate::app::server::AppState;
    use crate::app::server::convert::JobPhase;

    let state: AppState = expect_context();
    let jobs = state.jobs.read().await;
    let job = jobs
        .get(&job_id)
        .ok_or_else(|| ServerFnError::new("job not found"))?;

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
