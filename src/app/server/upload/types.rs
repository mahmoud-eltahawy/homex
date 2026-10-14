use std::path::PathBuf;

use tempfile::TempDir;
use uuid::Uuid;

#[derive(Debug, Clone)]
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

pub struct UploadPayload {
    pub section_slug: String,
    pub target: UploadTarget,
    pub season_number: Option<i64>,
    pub files: Vec<UploadFile>,
    /// Audio only — video posters are extracted from the media itself.
    pub poster: Option<(String, Vec<u8>)>,
    /// Auto-cleaning temp directory. Dropping the payload wipes its
    /// contents; there is no manual cleanup path.
    pub temp_dir: TempDir,
}

pub struct UploadFile {
    pub filename: String,
    pub title: String,
    pub temp_path: PathBuf,
    pub size: u64,
}

pub struct StagedFile {
    pub rel: String,
    pub duration: i64,
    pub size: u64,
    pub title: String,
    /// Where these bytes came from. Retained so a failure after staging
    /// can put the source back, which is what makes `retry_failed`
    /// actually retryable on the classify path.
    pub origin: std::path::PathBuf,
}
