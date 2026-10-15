//! Multipart form field names.
//!
//! Two wire formats share this table:
//!
//! * the **upload form** — `upload_media`, `new_collection`, `AppendItems`;
//! * the **poster-upload form** — `upload_collection_poster`.
//!
//! The `poster_file` name is used by both, so it lives on its own.

// ─── Shared ───────────────────────────────────────────────────────────────

pub const POSTER_FILE: &str = "poster_file";

// ─── Upload form ──────────────────────────────────────────────────────────

pub const SECTION_SLUG: &str = "section_slug";
pub const COLLECTION_ID: &str = "collection_id";
pub const COLLECTION_TITLE: &str = "collection_title";
pub const DESCRIPTION: &str = "description";
pub const SEASON_NUMBER: &str = "season_number";
pub const TAG_PREFIX: &str = "tag_";
pub const FILE_PREFIX: &str = "file_";
pub const FILE_TITLE_PREFIX: &str = "file_title_";

pub fn tag(idx: usize) -> String {
    format!("{TAG_PREFIX}{idx}")
}
pub fn file(idx: usize) -> String {
    format!("{FILE_PREFIX}{idx}")
}
pub fn file_title(idx: usize) -> String {
    format!("{FILE_TITLE_PREFIX}{idx}")
}

// ─── Poster-upload form ───────────────────────────────────────────────────

pub const POSTER_SECTION: &str = "section";
pub const POSTER_ID: &str = "id";
