// ─── Slug ─────────────────────────────────────────────────────────────────
//
// Re-exported so callers can keep using `super::naming::slugify`. The `slug`
// crate handles Unicode (via `deunicode`) — hand-rolled ASCII-only versions
// collapse non-Latin titles to an empty string.

pub use slug::slugify;

// ─── Storage token ────────────────────────────────────────────────────────
//
// Random v4 UUID, hex-encoded by `Display`. Collisions are astronomically
// unlikely and callers don't depend on ordering.

pub fn new_storage_token() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn new_job_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

// ─── Filename helpers ─────────────────────────────────────────────────────

pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| !matches!(c, '/' | '\\' | '\0'))
        .collect()
}

/// Extension for a poster upload. Falls back to `"jpg"` when the incoming
/// filename has no plausible extension.
///
/// This is only the *candidate* extension — the poster write path replaces
/// it with the magic-byte-detected extension when `infer` recognizes the
/// bytes. See `server::poster::resolve_poster_ext`.
pub fn extension_of(filename: &str) -> String {
    match filename.rsplit_once('.') {
        Some((_, ext))
            if !ext.is_empty()
                && ext.len() <= 8
                && ext.chars().all(|c| c.is_ascii_alphanumeric()) =>
        {
            ext.to_ascii_lowercase()
        }
        _ => "jpg".to_string(),
    }
}
