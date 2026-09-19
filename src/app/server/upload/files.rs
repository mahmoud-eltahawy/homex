use leptos::prelude::ServerFnError;

use crate::app::model::MediaType;
use crate::app::server::AppState;
use crate::app::server::convert::{
    TargetFormat, convert_file, ffprobe_duration, is_audio_container_supported,
    is_video_container_supported,
};

use super::naming::{sanitize_filename, slugify};
use super::types::{StagedFile, UploadPayload};

pub(super) fn base_subdir(media_type: MediaType) -> &'static str {
    match media_type {
        MediaType::Movie => "movies",
        MediaType::Series => "series",
        MediaType::AudioGroup => "audio",
    }
}

/// Which target format each file needs (`None` = already browser-playable).
fn targets_for(payload: &UploadPayload) -> Vec<Option<TargetFormat>> {
    payload
        .files
        .iter()
        .map(|f| {
            let ext = f.filename.rsplit('.').next().unwrap_or("");
            match payload.media_type {
                MediaType::Movie | MediaType::Series => {
                    if is_video_container_supported(ext) {
                        None
                    } else {
                        Some(TargetFormat::Mp4)
                    }
                }
                MediaType::AudioGroup => {
                    if is_audio_container_supported(ext) {
                        None
                    } else {
                        Some(TargetFormat::Mp3)
                    }
                }
            }
        })
        .collect()
}

pub async fn stage_files(
    payload: &UploadPayload,
    state: &AppState,
    job_id: Option<&str>,
) -> Result<Vec<StagedFile>, ServerFnError> {
    let subdir = base_subdir(payload.media_type);
    let slug = slugify(&payload.title);
    let base = state.config.storage.media_root.join(subdir).join(&slug);
    tokio::fs::create_dir_all(&base)
        .await
        .map_err(|e| ServerFnError::new(format!("mkdir: {e}")))?;

    let targets = targets_for(payload);
    let conversion_count = targets.iter().filter(|t| t.is_some()).count();
    let mut staged = Vec::with_capacity(payload.files.len());

    for (i, file) in payload.files.iter().enumerate() {
        let ext = file.filename.rsplit('.').next().unwrap_or("").to_string();
        let stem = file
            .filename
            .rsplitn(2, '.')
            .nth(1)
            .unwrap_or(&file.filename)
            .to_string();
        let safe_stem = sanitize_filename(&stem);

        let raw_rel = format!("{subdir}/{slug}/{safe_stem}.{ext}");
        let raw_abs = state.config.storage.media_root.join(&raw_rel);

        tokio::fs::write(&raw_abs, &file.bytes)
            .await
            .map_err(|e| ServerFnError::new(format!("write {}: {e}", raw_abs.display())))?;

        let Some(target) = targets[i] else {
            let dur = ffprobe_duration(&raw_abs).await.round() as i64;
            staged.push(StagedFile {
                rel: raw_rel,
                duration: dur,
                size: file.bytes.len() as u64,
                title: file.title.clone(),
            });
            continue;
        };

        let out_rel = format!("{subdir}/{slug}/{safe_stem}.{}", target.extension());
        let out_abs = state.config.storage.media_root.join(&out_rel);

        let conversion_pos = targets[..i].iter().filter(|t| t.is_some()).count();
        let total_secs = ffprobe_duration(&raw_abs).await;

        convert_file(
            &raw_abs,
            &out_abs,
            target,
            total_secs,
            conversion_pos,
            conversion_count,
            &file.filename,
            &state.jobs,
            job_id,
        )
        .await
        .map_err(ServerFnError::new)?;

        let _ = tokio::fs::remove_file(&raw_abs).await;

        let size = tokio::fs::metadata(&out_abs)
            .await
            .map(|m| m.len())
            .unwrap_or(0);

        staged.push(StagedFile {
            rel: out_rel,
            duration: total_secs.round() as i64,
            size,
            title: file.title.clone(),
        });
    }

    Ok(staged)
}
