//! Read-only helpers for walking the incoming media tree.

use std::path::{Path, PathBuf};

use crate::app::error::{AppError, ErrorContext};
use crate::app::model::MediaKind;
use crate::app::types::{FileName, RelPath};

#[derive(Debug, Clone)]
pub struct RawFile {
    pub abs: PathBuf,
    pub rel: RelPath,
    pub filename: FileName,
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
            let name_str = entry.file_name().to_string_lossy().to_string();
            if name_str.starts_with('.') {
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

                // Every file we reach here was discovered by walking from
                // `root_canon`, so it is guaranteed to be under that root.
                // A failure to strip the prefix or to build a valid `RelPath`
                // would mean an invariant of the walk itself is broken —
                // log and skip rather than crash.
                let Ok(rel_os) = abs.strip_prefix(root_canon) else {
                    leptos::logging::warn!("[incoming] path not under root: {}", abs.display(),);
                    continue;
                };
                let rel_string = rel_os.to_string_lossy().into_owned();
                let Ok(rel) = RelPath::new(rel_string) else {
                    leptos::logging::warn!(
                        "[incoming] invalid relative path: {}",
                        rel_os.display(),
                    );
                    continue;
                };
                let Ok(filename) = FileName::new(name_str) else {
                    leptos::logging::warn!("[incoming] invalid filename");
                    continue;
                };

                out.push(RawFile {
                    abs,
                    rel,
                    filename,
                    size,
                });
            }
        }
    }
    out.sort_by(|a, b| {
        a.rel
            .as_str()
            .to_lowercase()
            .cmp(&b.rel.as_str().to_lowercase())
    });
    Ok(out)
}

pub fn dominant_kind(files: &[RawFile]) -> Option<MediaKind> {
    let first = files
        .first()
        .and_then(|f| MediaKind::from_filename(f.filename.as_str()))?;
    for f in &files[1..] {
        if MediaKind::from_filename(f.filename.as_str()) != Some(first) {
            return None;
        }
    }
    Some(first)
}

pub fn ensure_kind_matches(files: &[RawFile], kind: MediaKind) -> Result<(), AppError> {
    for f in files {
        if MediaKind::from_filename(f.filename.as_str()) != Some(kind) {
            return Err(AppError::detailed(format!(
                "File '{}' is not {} media",
                f.filename,
                kind.as_str()
            )));
        }
    }
    Ok(())
}
