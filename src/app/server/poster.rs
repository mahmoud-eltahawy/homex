use crate::app::constants::{routes, storage};
use leptos::prelude::ServerFnError;
use std::path::Path;

const ALLOWED_POSTER_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif"];

const MAX_POSTER_BYTES: usize = 5 * 1024 * 1024;

/// Resolve the canonical extension for a poster upload.
///
/// `infer` checks the magic bytes first — this defends against a client
/// that names a non-image as `.jpg`, and against a client that names a
/// PNG as `.jpeg`. Falls back to the client-supplied extension only if
/// `infer` can't classify the bytes at all.
fn resolve_poster_ext(client_ext: &str, bytes: &[u8]) -> Result<&'static str, ServerFnError> {
    let candidate = match infer::get(bytes) {
        Some(kind) => kind.extension().to_string(),
        None => client_ext.to_ascii_lowercase(),
    };

    ALLOWED_POSTER_EXTS
        .iter()
        .find(|allowed| allowed.eq_ignore_ascii_case(&candidate))
        .copied()
        .ok_or_else(|| ServerFnError::new(format!("Unsupported image format: .{candidate}")))
}

pub async fn write_poster(
    data_dir: &Path,
    subdir: &str,
    entity_id: i64,
    client_ext: &str,
    bytes: &[u8],
) -> Result<String, ServerFnError> {
    if bytes.len() > MAX_POSTER_BYTES {
        return Err(ServerFnError::new(format!(
            "Image size exceeds {} MB",
            MAX_POSTER_BYTES / 1024 / 1024
        )));
    }

    let ext = resolve_poster_ext(client_ext, bytes)?;

    let dir = data_dir.join(storage::POSTERS_DIR).join(subdir);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| ServerFnError::new(format!("mkdir posters: {e}")))?;

    let filename = format!("{entity_id}.{ext}");
    tokio::fs::write(dir.join(&filename), bytes)
        .await
        .map_err(|e| ServerFnError::new(format!("write poster: {e}")))?;

    Ok(routes::poster(subdir, &filename))
}

/// Best-effort move of a poster between two section subdirectories. Returns
/// the new URL on success, `None` if the file couldn't be moved (the caller
/// then leaves the DB URL as-is, which keeps the poster working from the old
/// path).
pub async fn move_poster_to_subdir(
    data_dir: &Path,
    from_subdir: &str,
    to_subdir: &str,
    poster_url: &str,
) -> Option<String> {
    let rest = poster_url.strip_prefix(routes::POSTERS_URL_PREFIX)?;
    let (_old, filename) = rest.split_once('/')?;

    let to_dir = data_dir.join(storage::POSTERS_DIR).join(to_subdir);
    if tokio::fs::create_dir_all(&to_dir).await.is_err() {
        return None;
    }

    let from_path = data_dir
        .join(storage::POSTERS_DIR)
        .join(from_subdir)
        .join(filename);
    let to_path = to_dir.join(filename);

    let moved = match tokio::fs::rename(&from_path, &to_path).await {
        Ok(()) => true,
        Err(_) => {
            let copied = tokio::fs::copy(&from_path, &to_path).await.is_ok();
            if copied {
                let _ = tokio::fs::remove_file(&from_path).await;
            }
            copied
        }
    };
    if !moved {
        return None;
    }

    Some(format!(
        "{}{}/{}",
        routes::POSTERS_URL_PREFIX,
        to_subdir,
        filename
    ))
}
