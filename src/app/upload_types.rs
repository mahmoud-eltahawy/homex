//! Shared types and constants for the upload path.
#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::ids::Slug;
use crate::app::types::{FileExt, TagName};

// ─── Container classification ─────────────────────────────────────────────

pub const VIDEO_SUPPORTED_EXTS: &[&str] = &["mp4", "m4v", "webm"];
pub const AUDIO_SUPPORTED_EXTS: &[&str] = &["mp3", "m4a", "aac", "wav", "ogg", "oga", "opus"];
pub const VIDEO_CONVERTIBLE_EXTS: &[&str] = &[
    "mkv", "mov", "avi", "wmv", "flv", "ts", "mpg", "mpeg", "3gp", "ogv",
];
pub const AUDIO_CONVERTIBLE_EXTS: &[&str] = &["flac", "wma", "aiff", "aif", "alac", "ape"];

fn in_list(list: &[&str], ext: &FileExt) -> bool {
    list.iter().any(|e| *e == ext.as_str())
}

pub fn is_video_container_supported(ext: &FileExt) -> bool {
    in_list(VIDEO_SUPPORTED_EXTS, ext)
}
pub fn is_audio_container_supported(ext: &FileExt) -> bool {
    in_list(AUDIO_SUPPORTED_EXTS, ext)
}
pub fn is_convertible_video(ext: &FileExt) -> bool {
    in_list(VIDEO_CONVERTIBLE_EXTS, ext)
}
pub fn is_convertible_audio(ext: &FileExt) -> bool {
    in_list(AUDIO_CONVERTIBLE_EXTS, ext)
}

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

pub const MAX_POSTER_BYTES: usize = 5 * 1024 * 1024;

// ─── Multipart field names ────────────────────────────────────────────────

pub const FORM_FIELD: &str = "form";
pub const FILE_FIELD: &str = "file";
pub const POSTER_FIELD: &str = "poster_file";

// ─── Wire types ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UploadTarget {
    Existing {
        collection_id: Uuid,
    },
    New {
        id: Uuid,
        title: String,
        description: Option<String>,
        tags: Vec<TagName>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadFileMeta {
    pub filename: String,
    pub title: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadForm {
    pub section_slug: Slug,
    pub target: UploadTarget,
    pub season_number: Option<i64>,
    pub files: Vec<UploadFileMeta>,
    pub has_poster: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PosterUploadForm {
    pub section_slug: Slug,
    pub collection_id: Uuid,
}
