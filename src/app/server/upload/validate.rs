use leptos::prelude::ServerFnError;

use super::types::UploadPayload;
use crate::app::model::MediaKind;
use crate::app::server::convert::{
    is_audio_container_supported, is_convertible_audio, is_convertible_video,
    is_video_container_supported,
};

pub fn validate_extensions(payload: &UploadPayload, kind: MediaKind) -> Result<(), ServerFnError> {
    for f in &payload.files {
        let ext = f
            .filename
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        let ok = match kind {
            MediaKind::Video => is_video_container_supported(&ext) || is_convertible_video(&ext),
            MediaKind::Audio => is_audio_container_supported(&ext) || is_convertible_audio(&ext),
        };
        if !ok {
            return Err(ServerFnError::new(format!(
                "Unsupported format: {}",
                f.filename
            )));
        }
    }
    Ok(())
}

pub fn needs_conversion(payload: &UploadPayload, kind: MediaKind) -> bool {
    payload.files.iter().any(|f| {
        let ext = f.filename.rsplit('.').next().unwrap_or("");
        match kind {
            MediaKind::Video => !is_video_container_supported(ext),
            MediaKind::Audio => !is_audio_container_supported(ext),
        }
    })
}
