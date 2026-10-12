use std::path::PathBuf;

use tempfile::TempDir;

pub struct UploadPayload {
    pub section_slug: String,
    pub collection_id: i64,
    pub title: String,
    pub description: String,
    pub season_number: Option<i64>,
    pub files: Vec<UploadFile>,
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
}
