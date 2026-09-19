use leptos::prelude::ServerFnError;

use crate::app::model::MediaType;
use crate::app::server::convert::{
    is_audio_container_supported, is_convertible_audio, is_convertible_video,
    is_video_container_supported,
};

use super::types::UploadPayload;

pub fn validate_extensions(payload: &UploadPayload) -> Result<(), ServerFnError> {
    for file in &payload.files {
        let ext = file
            .filename
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();

        let ok = match payload.media_type {
            MediaType::Movie | MediaType::Series => {
                is_video_container_supported(&ext) || is_convertible_video(&ext)
            }
            MediaType::AudioGroup => {
                is_audio_container_supported(&ext) || is_convertible_audio(&ext)
            }
        };

        if !ok {
            return Err(ServerFnError::new(format!(
                "صيغة غير مدعومة: {}",
                file.filename
            )));
        }
    }
    Ok(())
}

pub fn needs_conversion(payload: &UploadPayload) -> bool {
    payload.files.iter().any(|f| {
        let ext = f.filename.rsplit('.').next().unwrap_or("");
        match payload.media_type {
            MediaType::Movie | MediaType::Series => !is_video_container_supported(ext),
            MediaType::AudioGroup => !is_audio_container_supported(ext),
        }
    })
}
