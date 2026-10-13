//! Reverse of the staging pipeline: move a file out of `media_root` and back
//! into the incoming directory so it can be re-classified.

use std::path::{Path, PathBuf};

use uuid::Uuid;

use crate::app::error::AppError;

/// Move one file back into the incoming directory.
///
/// Returns `Ok(Some(new_path))` if the file was moved, `Ok(None)` if it was
/// skipped (external URL, missing file, path outside `media_root`). `Err`
/// only for genuinely unexpected failures like a cross-device copy error.
pub async fn move_file_to_incoming(
    media_root: &Path,
    incoming_dir: &Path,
    relative_path: &str,
) -> Result<Option<PathBuf>, AppError> {
    if relative_path.starts_with("http://") || relative_path.starts_with("https://") {
        return Ok(None);
    }
    let abs = media_root.join(relative_path);
    let abs_canon = match tokio::fs::canonicalize(&abs).await {
        Ok(p) => p,
        Err(_) => return Ok(None),
    };
    if !abs_canon.starts_with(media_root) {
        return Ok(None);
    }

    let name = abs_canon
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let dest = unique_incoming_path(incoming_dir, &name).await;

    if tokio::fs::rename(&abs_canon, &dest).await.is_err() {
        tokio::fs::copy(&abs_canon, &dest)
            .await
            .map_err(|e| AppError::internal("move to incoming", e))?;
        let _ = tokio::fs::remove_file(&abs_canon).await;
    }
    Ok(Some(dest))
}

/// Pick a destination name in `dir` that doesn't collide with an existing
/// file: `foo.mp4`, `foo (2).mp4`, `foo (3).mp4`, …
async fn unique_incoming_path(dir: &Path, name: &str) -> PathBuf {
    let initial = dir.join(name);
    if tokio::fs::metadata(&initial).await.is_err() {
        return initial;
    }
    let (stem, ext) = match name.rsplit_once('.') {
        Some((s, e)) => (s.to_string(), format!(".{e}")),
        None => (name.to_string(), String::new()),
    };
    for n in 2..10_000 {
        let candidate = dir.join(format!("{stem} ({n}){ext}"));
        if tokio::fs::metadata(&candidate).await.is_err() {
            return candidate;
        }
    }
    dir.join(format!("{stem}-{}{ext}", Uuid::new_v4()))
}
