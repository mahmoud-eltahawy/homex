use crate::app::model::MediaType;

/// Everything the upload pipeline needs, already extracted from multipart.
/// Constructed by the `#[server] upload_media` body in `crate::app::upload`.
pub struct UploadPayload {
    pub title: String,
    pub description: String,
    pub media_type: MediaType,
    pub is_new: bool,
    pub existing_id: Option<i64>,
    pub season_number: Option<i64>,
    pub files: Vec<UploadFile>,
    /// `Some((extension, bytes))` if the user attached a poster.
    pub poster: Option<(String, Vec<u8>)>,
}

pub struct UploadFile {
    pub filename: String,
    pub title: String,
    pub bytes: Vec<u8>,
}

/// A file that has been written to disk and (if necessary) transcoded.
/// `rel` is the media-root-relative path stored in `files.relative_path`.
pub struct StagedFile {
    pub rel: String,
    pub duration: i64,
    pub size: u64,
    pub title: String,
}
