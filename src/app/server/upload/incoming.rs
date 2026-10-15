//! Read-only helpers for walking the incoming media tree.

use std::path::{Path, PathBuf};

use crate::app::error::{AppError, ErrorContext};
use crate::app::model::MediaKind;

#[derive(Debug, Clone)]
pub struct RawFile {
    pub abs: PathBuf,
    pub rel: String,
    pub filename: String,
    pub size: u64,
}

pub async fn read_immediate_subdirs(abs: &Path) -> Result<Vec<String>, AppError> {
    let mut rd = tokio::fs::read_dir(abs)
        .await
        .map_err(|e| AppError::detailed(format!("Cannot read folder: {e}")))?;
    let mut names = Vec::new();
    while let Some(entry) = rd.next_entry().await.context("read dir entry")? {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        if entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false) {
            names.push(name);
        }
    }
    names.sort_by_key(|s| s.to_lowercase());
    Ok(names)
}

pub async fn collect_files(
    root_canon: &Path,
    abs: &Path,
    recursive: bool,
) -> Result<Vec<RawFile>, AppError> {
    let mut stack = vec![abs.to_path_buf()];
    let mut out = Vec::new();
    while let Some(dir) = stack.pop() {
        let mut rd = match tokio::fs::read_dir(&dir).await {
            Ok(r) => r,
            Err(_) => continue,
        };
        while let Some(entry) = rd.next_entry().await.context("read dir entry")? {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            let ft = match entry.file_type().await {
                Ok(t) => t,
                Err(_) => continue,
            };
            if ft.is_symlink() {
                continue;
            }
            if ft.is_dir() {
                if recursive {
                    stack.push(entry.path());
                }
            } else if ft.is_file() {
                let size = entry.metadata().await.map(|m| m.len()).unwrap_or(0);
                let abs = entry.path();
                let rel = abs
                    .strip_prefix(root_canon)
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|_| name.clone());
                out.push(RawFile {
                    abs,
                    rel,
                    filename: name,
                    size,
                });
            }
        }
    }
    out.sort_by_key(|f| f.rel.to_lowercase());
    Ok(out)
}

pub fn dominant_kind(files: &[RawFile]) -> Option<MediaKind> {
    let first = files
        .first()
        .and_then(|f| MediaKind::from_filename(&f.filename))?;
    for f in &files[1..] {
        if MediaKind::from_filename(&f.filename) != Some(first) {
            return None;
        }
    }
    Some(first)
}

pub fn ensure_kind_matches(files: &[RawFile], kind: MediaKind) -> Result<(), AppError> {
    for f in files {
        if MediaKind::from_filename(&f.filename) != Some(kind) {
            return Err(AppError::detailed(format!(
                "File '{}' is not {} media",
                f.filename,
                kind.as_str()
            )));
        }
    }
    Ok(())
}
