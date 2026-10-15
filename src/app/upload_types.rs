//! Shared types and constants for the upload path.
//!
//! Everything here is visible to both client and server: the same constant
//! drives which extensions the `<input type="file">` offers and which
//! extensions the server accepts. Two lists that can't disagree.
//!
//! The `is_*` classifiers below are also re-exported through
//! `crate::app::server::convert`, so call sites that predate this module
//! keep working. Until that re-export lands, `dead_code` is silenced here
//! because the module is a shared API surface and some items are only
//! referenced from server-only paths.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ─── Container classification ─────────────────────────────────────────────

pub const VIDEO_SUPPORTED_EXTS: &[&str] = &["mp4", "m4v", "webm"];
pub const AUDIO_SUPPORTED_EXTS: &[&str] = &["mp3", "m4a", "aac", "wav", "ogg", "oga", "opus"];
pub const VIDEO_CONVERTIBLE_EXTS: &[&str] = &[
    "mkv", "mov", "avi", "wmv", "flv", "ts", "mpg", "mpeg", "3gp", "ogv",
];
pub const AUDIO_CONVERTIBLE_EXTS: &[&str] = &["flac", "wma", "aiff", "aif", "alac", "ape"];

fn in_list(list: &[&str], ext: &str) -> bool {
    list.iter().any(|e| e.eq_ignore_ascii_case(ext))
}

pub fn is_video_container_supported(ext: &str) -> bool {
    in_list(VIDEO_SUPPORTED_EXTS, ext)
}
pub fn is_audio_container_supported(ext: &str) -> bool {
    in_list(AUDIO_SUPPORTED_EXTS, ext)
}
pub fn is_convertible_video(ext: &str) -> bool {
    in_list(VIDEO_CONVERTIBLE_EXTS, ext)
}
pub fn is_convertible_audio(ext: &str) -> bool {
    in_list(AUDIO_CONVERTIBLE_EXTS, ext)
}

/// Build an `accept` attribute for `<input type="file">` that matches the
/// server's allowlist exactly. Both "kept as-is" and "converted" extensions
/// are included because the picker should let the user choose either.
pub fn video_accept_attr() -> String {
    VIDEO_SUPPORTED_EXTS
        .iter()
        .chain(VIDEO_CONVERTIBLE_EXTS)
        .map(|e| format!(".{e}"))
        .collect::<Vec<_>>()
        .join(",")
}
pub fn audio_accept_attr() -> String {
    AUDIO_SUPPORTED_EXTS
        .iter()
        .chain(AUDIO_CONVERTIBLE_EXTS)
        .map(|e| format!(".{e}"))
        .collect::<Vec<_>>()
        .join(",")
}

// ─── Limits ───────────────────────────────────────────────────────────────

pub const MAX_POSTER_BYTES: usize = 5 * 1024 * 1024;

// ─── Multipart field names ────────────────────────────────────────────────
//
// Two wire formats:
//
//   * the **media-upload form** sends one `form` field (JSON) followed by
//     zero or more `file` fields and optionally one `poster_file`;
//   * the **poster-upload form** sends one `form` field (JSON) followed by
//     one `poster_file`.

pub const FORM_FIELD: &str = "form";
pub const FILE_FIELD: &str = "file";
pub const POSTER_FIELD: &str = "poster_file";

// ─── Wire types ───────────────────────────────────────────────────────────

/// Where an upload is destined.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UploadTarget {
    Existing {
        collection_id: Uuid,
    },
    New {
        id: Uuid,
        title: String,
        description: Option<String>,
        tags: Vec<String>,
    },
}

/// Per-file metadata that accompanies each raw `file` blob in the stream.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadFileMeta {
    pub filename: String,
    pub title: String,
}

/// The JSON document that opens a media-upload multipart request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadForm {
    pub section_slug: String,
    pub target: UploadTarget,
    pub season_number: Option<i64>,
    pub files: Vec<UploadFileMeta>,
    pub has_poster: bool,
}

/// The JSON document that opens a poster-upload multipart request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PosterUploadForm {
    pub section_slug: String,
    pub collection_id: Uuid,
}
