use std::sync::atomic::{AtomicU64, Ordering};

static STORAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Short, collision-resistant suffix used to disambiguate on-disk filenames.
/// Mixes wall-clock nanoseconds with a process-local counter so two files
/// staged in the same nanosecond (or in concurrent uploads) still differ.
pub fn new_storage_token() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let c = STORAGE_COUNTER.fetch_add(1, Ordering::Relaxed);

    format!("{:010x}{:04x}", ns & 0xFFFFFFFFFF, c & 0xFFFF)
}

pub fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_dash = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| !matches!(c, '/' | '\\' | '\0'))
        .collect()
}

/// Extension for a poster upload. Falls back to `"jpg"` when the incoming
/// filename has no plausible extension.
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

pub fn new_job_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{ns:x}")
}
