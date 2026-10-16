use std::path::PathBuf;

use tempfile::TempDir;

use crate::app::ids::Slug;
use crate::app::types::{FileExt, FileName, RelPath, Title};

pub use crate::app::upload_types::UploadTarget;

pub struct UploadPayload {
    pub section_slug: Slug,
    pub target: UploadTarget,
    pub season_number: Option<i64>,
    pub files: Vec<UploadFile>,
    /// Audio only — video posters are extracted from the media itself.
    ///
    /// The extension is a [`FileExt`] rather than a `String` so the value
    /// that reaches `write_poster` is already lowercased, alphanumeric, and
    /// bounded in length.
    pub poster: Option<(FileExt, Vec<u8>)>,
    /// Auto-cleaning temp directory.
    pub temp_dir: TempDir,
}

pub struct UploadFile {
    pub filename: FileName,
    pub title: Title,
    pub temp_path: PathBuf,
    pub size: u64,
}

pub struct StagedFile {
    pub rel: RelPath,
    pub duration: i64,
    pub size: u64,
    pub title: Title,
    pub origin: PathBuf,
}
