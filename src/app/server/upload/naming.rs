//! Filename and path helpers for the upload pipeline.

pub use slug::slugify;

use crate::app::error::{AppError, ErrorCode};
use crate::app::types::{FileExt, RelPath};

// ─── Filename helpers ─────────────────────────────────────────────────────

pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| !matches!(c, '/' | '\\' | '\0'))
        .collect()
}

/// Candidate extension for a poster upload.
///
/// Falls back to `"jpg"` when the incoming filename has no plausible
/// extension. Only the *candidate* — the poster write path replaces it with
/// the magic-byte-detected extension when `infer` recognizes the bytes. See
/// `server::poster::write_poster`.
pub fn extension_of(filename: &str) -> FileExt {
    FileExt::from_filename(filename)
        .unwrap_or_else(|| FileExt::new("jpg").expect("`jpg` is a valid extension"))
}

/// Validate a relative path.
pub fn sanitize_rel(rel: &str) -> Result<RelPath, AppError> {
    RelPath::new(rel).map_err(|_| AppError::public(ErrorCode::InvalidPath))
}
