use crate::app::error::{AppError, ErrorCode};
use crate::app::model::MediaKind;
use crate::app::types::FileExt;
use crate::app::upload_types::{
    is_audio_container_supported, is_convertible_audio, is_convertible_video,
    is_video_container_supported,
};

use super::types::UploadPayload;

pub fn validate_extensions(payload: &UploadPayload, kind: MediaKind) -> Result<(), AppError> {
    for f in &payload.files {
        let Some(ext) = FileExt::from_filename(&f.filename) else {
            return Err(AppError::public(ErrorCode::UnsupportedFormat));
        };
        let ok = match kind {
            MediaKind::Video => is_video_container_supported(&ext) || is_convertible_video(&ext),
            MediaKind::Audio => is_audio_container_supported(&ext) || is_convertible_audio(&ext),
        };
        if !ok {
            return Err(AppError::public(ErrorCode::UnsupportedFormat));
        }
    }
    Ok(())
}

pub fn needs_conversion(payload: &UploadPayload, kind: MediaKind) -> bool {
    payload.files.iter().any(|f| {
        let Some(ext) = FileExt::from_filename(&f.filename) else {
            return true;
        };
        match kind {
            MediaKind::Video => !is_video_container_supported(&ext),
            MediaKind::Audio => !is_audio_container_supported(&ext),
        }
    })
}
