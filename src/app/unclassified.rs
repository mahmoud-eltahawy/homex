//! Unclassified media browser.
//!
//! Exposes the optional incoming folder (`HOMEX_UNCLASSIFIED_DIR`) as a
//! navigateable tree. Files, batches of files, and whole folders can be
//! classified into the library.
//!
//! ## Parallel classification
//!
//! Every file in a classify request becomes its own independent job running
//! in its own `tokio::spawn` task. The client tracks all jobs by id and shows
//! one progress bar per file.
//!
//! Concurrency is bounded by a process-wide semaphore. Each job has a hard
//! timeout and its own `CancellationToken`. The client can retry failed
//! files without reselecting them.
//!
//! ## URL as source of truth
//!
//! The current folder is encoded in the URL as `?path=<relative>`.

use gloo_timers::future::TimeoutFuture;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_query_map};
use leptos_router::{LazyRoute, lazy_route};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::collections::create_collection_named;
use crate::app::constants::routes;
use crate::app::error::AppError;
use crate::app::error_view::{ErrorStyle, ErrorView, action_error};
use crate::app::model::{CollectionSummary, MediaKind};
use crate::app::resource_view::ResourceView;
use crate::app::upload_api::ConversionStatus;

#[cfg(feature = "ssr")]
use crate::app::error::ErrorContext;
#[cfg(feature = "ssr")]
use std::time::Duration;
#[cfg(feature = "ssr")]
use tokio_util::sync::CancellationToken;

// ══════════════════════════════════════════════════════════════════════════════
//  Constants
// ══════════════════════════════════════════════════════════════════════════════

const POLL_INTERVAL_MS: u32 = 700;
const MAX_CONSECUTIVE_ERRORS: u32 = 10;

#[cfg(feature = "ssr")]
const JOB_TIMEOUT_SECS: u64 = 15 * 60;
#[cfg(feature = "ssr")]
const WATCHER_DEADLINE_SECS: u64 = 30 * 60;

const VIDEO_EXTS: &[&str] = &[
    "mp4", "mkv", "mov", "webm", "avi", "m4v", "wmv", "flv", "ts", "mpg", "mpeg", "3gp", "ogv",
];
const AUDIO_EXTS: &[&str] = &[
    "mp3", "m4a", "aac", "wav", "ogg", "oga", "opus", "flac", "wma", "aiff", "aif", "alac", "ape",
];

// ══════════════════════════════════════════════════════════════════════════════
//  Wire types
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnclassifiedEntry {
    pub name: String,
    pub rel_path: String,
    pub is_dir: bool,
    pub size: u64,
    pub ext: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionWithCollections {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub nested: bool,
    pub collections: Vec<CollectionSummary>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClassifyInput {
    pub rel_path: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderInspection {
    pub rel_path: String,
    pub name: String,
    pub has_subdirs: bool,
    pub subdir_names: Vec<String>,
    pub file_count: usize,
    pub total_size: u64,
    pub dominant_kind: Option<MediaKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoldersInspection {
    pub folders: Vec<FolderInspection>,
    pub total_files: usize,
    pub total_size: u64,
    pub any_has_subdirs: bool,
    /// `Some` when every folder shares one media kind; `None` for a mixed set.
    pub common_kind: Option<MediaKind>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FolderDestination {
    Existing { section_slug: String },
    New { section_title: String },
}

/// One file's job.
///
/// `rel_path` / `title` / `season_number` / `collection_id` let the client
/// reconstruct a retry request for a failed file without asking the user to
/// reselect anything.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileJobHandle {
    pub name: String,
    pub job_id: String,
    pub section_slug: String,
    pub collection_id: Option<Uuid>,
    pub rel_path: String,
    pub title: String,
    pub season_number: Option<i64>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DestinationMode {
    Existing,
    New,
}

// ══════════════════════════════════════════════════════════════════════════════
//  Pure helpers
// ══════════════════════════════════════════════════════════════════════════════

fn kind_for_ext(ext: Option<&str>) -> Option<MediaKind> {
    let ext = ext?;
    if VIDEO_EXTS.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
        Some(MediaKind::Video)
    } else if AUDIO_EXTS.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
        Some(MediaKind::Audio)
    } else {
        None
    }
}

#[cfg(feature = "ssr")]
fn kind_for_name(name: &str) -> Option<MediaKind> {
    kind_for_ext(name.rsplit_once('.').map(|(_, e)| e))
}

fn format_size(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}

fn file_stem(name: &str) -> String {
    name.rsplit_once('.')
        .map(|(s, _)| s.to_string())
        .unwrap_or_else(|| name.to_string())
}

fn encode_query_path(s: &str) -> String {
    use std::fmt::Write;
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                out.push(b as char);
            }
            _ => {
                let _ = write!(out, "%{b:02X}");
            }
        }
    }
    out
}

fn select_all(selection: RwSignal<Vec<UnclassifiedEntry>>, entries: Vec<UnclassifiedEntry>) {
    selection.update(|v| {
        let target_kind = v
            .first()
            .and_then(|e| kind_for_ext(e.ext.as_deref()))
            .or_else(|| entries.iter().find_map(|e| kind_for_ext(e.ext.as_deref())));
        v.clear();
        for e in entries {
            let k = kind_for_ext(e.ext.as_deref());
            if k.is_none() || (target_kind.is_some() && k != target_kind) {
                continue;
            }
            v.push(e);
        }
    });
}

fn toggle_selection(selection: RwSignal<Vec<UnclassifiedEntry>>, entry: UnclassifiedEntry) {
    selection.update(|v| {
        if let Some(pos) = v.iter().position(|e| e.rel_path == entry.rel_path) {
            v.remove(pos);
            return;
        }
        let new_kind = kind_for_ext(entry.ext.as_deref());
        let existing_kind = v.first().and_then(|e| kind_for_ext(e.ext.as_deref()));
        if existing_kind.is_some() && existing_kind != new_kind {
            v.clear();
        }
        v.push(entry);
    });
}

fn toggle_folder_selection(selection: RwSignal<Vec<UnclassifiedEntry>>, entry: UnclassifiedEntry) {
    selection.update(|v| {
        if let Some(pos) = v.iter().position(|e| e.rel_path == entry.rel_path) {
            v.remove(pos);
        } else {
            v.push(entry);
        }
    });
}

fn parse_season(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.is_empty() { None } else { s.parse().ok() }
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

// ══════════════════════════════════════════════════════════════════════════════
//  Server-only filesystem helpers
// ══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "ssr")]
fn sanitize_rel(rel: &str) -> Result<String, AppError> {
    let trimmed = rel.trim_matches('/');
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    for part in trimmed.split('/') {
        if part.is_empty() || part == "." || part == ".." {
            return Err(AppError::public("Invalid path"));
        }
    }
    Ok(trimmed.to_string())
}

#[cfg(feature = "ssr")]
#[derive(Debug, Clone)]
struct RawFile {
    abs: std::path::PathBuf,
    rel: String,
    filename: String,
    size: u64,
}

#[cfg(feature = "ssr")]
async fn read_immediate_subdirs(abs: &std::path::Path) -> Result<Vec<String>, AppError> {
    let mut rd = tokio::fs::read_dir(abs)
        .await
        .map_err(|e| AppError::public(format!("Cannot read folder: {e}")))?;
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

#[cfg(feature = "ssr")]
async fn collect_files(
    root_canon: &std::path::Path,
    abs: &std::path::Path,
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

#[cfg(feature = "ssr")]
fn dominant_kind(files: &[RawFile]) -> Option<MediaKind> {
    let first = files.first().and_then(|f| kind_for_name(&f.filename))?;
    for f in &files[1..] {
        if kind_for_name(&f.filename) != Some(first) {
            return None;
        }
    }
    Some(first)
}

#[cfg(feature = "ssr")]
fn ensure_kind_matches(files: &[RawFile], kind: MediaKind) -> Result<(), AppError> {
    for f in files {
        if kind_for_name(&f.filename) != Some(kind) {
            return Err(AppError::public(format!(
                "File '{}' is not {} media",
                f.filename,
                kind.as_str()
            )));
        }
    }
    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════════
//  Concurrency, cancel and config guards
// ══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "ssr")]
fn upload_semaphore() -> &'static tokio::sync::Semaphore {
    use std::sync::OnceLock;
    use tokio::sync::Semaphore;
    static SEM: OnceLock<Semaphore> = OnceLock::new();
    SEM.get_or_init(|| {
        let permits = std::thread::available_parallelism()
            .map(|n| n.get() * 2)
            .unwrap_or(4)
            .clamp(2, 16);
        Semaphore::new(permits)
    })
}

#[cfg(feature = "ssr")]
fn canonical_media_root(
    media_root: &std::path::Path,
) -> Result<&'static std::path::Path, AppError> {
    use std::sync::OnceLock;
    static CACHE: OnceLock<Result<&'static std::path::PathBuf, String>> = OnceLock::new();
    let result = CACHE.get_or_init(|| {
        std::fs::canonicalize(media_root)
            .map(|p| Box::leak(Box::new(p)) as &'static std::path::PathBuf)
            .map_err(|e| format!("canonicalize media_root: {e}"))
    });
    match result {
        Ok(p) => Ok(p.as_path()),
        Err(e) => Err(AppError::internal("config", e.clone())),
    }
}

#[cfg(feature = "ssr")]
fn ensure_roots_disjoint(
    unclassified_canon: &std::path::Path,
    media_root: &std::path::Path,
) -> Result<(), AppError> {
    let media_canon = canonical_media_root(media_root)?;
    if unclassified_canon == media_canon
        || unclassified_canon.starts_with(media_canon)
        || media_canon.starts_with(unclassified_canon)
    {
        leptos::logging::error!(
            "[config] unclassified_dir ({}) overlaps media_root ({})",
            unclassified_canon.display(),
            media_canon.display(),
        );
        return Err(AppError::internal(
            "config",
            "unclassified_dir and media_root must not overlap",
        ));
    }
    Ok(())
}

#[cfg(feature = "ssr")]
fn active_classifications()
-> &'static std::sync::Mutex<std::collections::HashSet<std::path::PathBuf>> {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};
    static SET: OnceLock<Mutex<HashSet<std::path::PathBuf>>> = OnceLock::new();
    SET.get_or_init(|| Mutex::new(HashSet::new()))
}

#[cfg(feature = "ssr")]
struct ClassifyLocks(Vec<std::path::PathBuf>);

#[cfg(feature = "ssr")]
impl Drop for ClassifyLocks {
    fn drop(&mut self) {
        let mut set = active_classifications()
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        for p in &self.0 {
            set.remove(p);
        }
    }
}

#[cfg(feature = "ssr")]
fn acquire_classify_locks(paths: Vec<std::path::PathBuf>) -> Result<ClassifyLocks, AppError> {
    if paths.is_empty() {
        return Ok(ClassifyLocks(Vec::new()));
    }
    let mut set = active_classifications()
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    for p in &paths {
        if set.contains(p) {
            return Err(AppError::public(
                "A classification is already running for one of these paths",
            ));
        }
    }
    for p in &paths {
        set.insert(p.clone());
    }
    drop(set);
    Ok(ClassifyLocks(paths))
}

#[cfg(feature = "ssr")]
async fn existing_collection_titles(
    db: &mut toasty::Db,
    section_slug: &str,
) -> Result<std::collections::HashMap<String, Uuid>, AppError> {
    use crate::app::model::CollectionRow;
    use std::collections::HashMap;
    let rows = CollectionRow::filter(
        CollectionRow::fields()
            .section_slug()
            .eq(section_slug.to_string()),
    )
    .exec(db)
    .await
    .context("load existing collections")?;
    Ok(rows
        .into_iter()
        .map(|r| (r.title, r.id))
        .collect::<HashMap<_, _>>())
}

// ══════════════════════════════════════════════════════════════════════════════
//  Server functions — listing & inspection
// ══════════════════════════════════════════════════════════════════════════════

#[server]
pub async fn list_unclassified(rel_path: String) -> Result<Vec<UnclassifiedEntry>, AppError> {
    use crate::app::server::AppState;

    let state: AppState = expect_context();
    let root = state
        .config
        .storage
        .unclassified_dir
        .clone()
        .ok_or_else(|| AppError::public("Unclassified media folder is not configured"))?;

    let root_canon = tokio::fs::canonicalize(&root)
        .await
        .context("canonicalize unclassified root")?;
    ensure_roots_disjoint(&root_canon, &state.config.storage.media_root)?;

    let clean = sanitize_rel(&rel_path)?;
    let abs = if clean.is_empty() {
        root.clone()
    } else {
        root.join(&clean)
    };

    let abs_canon = tokio::fs::canonicalize(&abs)
        .await
        .map_err(|e| AppError::public(format!("Cannot open folder: {e}")))?;
    if !abs_canon.starts_with(&root_canon) {
        return Err(AppError::public("Invalid path"));
    }

    let mut rd = tokio::fs::read_dir(&abs_canon)
        .await
        .map_err(|e| AppError::public(format!("Cannot read folder: {e}")))?;

    let mut entries = Vec::new();
    while let Some(entry) = rd.next_entry().await.context("read dir entry")? {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let meta = match entry.metadata().await {
            Ok(m) => m,
            Err(_) => continue,
        };
        let is_dir = meta.is_dir();
        entries.push(UnclassifiedEntry {
            size: if is_dir { 0 } else { meta.len() },
            ext: if is_dir { None } else { ext_of(&name) },
            rel_path: if clean.is_empty() {
                name.clone()
            } else {
                format!("{clean}/{name}")
            },
            name,
            is_dir,
        });
    }

    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(entries)
}

#[cfg(feature = "ssr")]
fn ext_of(name: &str) -> Option<String> {
    name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase())
}

#[server]
pub async fn fetch_sections_with_collections(
    media_kind: String,
) -> Result<Vec<SectionWithCollections>, AppError> {
    use crate::app::model::{CollectionRow, SectionRow};
    use crate::app::server::AppState;

    let mut state: AppState = expect_context();

    let sections = SectionRow::filter(SectionRow::fields().media_kind().eq(media_kind))
        .order_by(SectionRow::fields().title().asc())
        .exec(&mut state.db)
        .await
        .context("fetch sections")?;

    let mut out = Vec::with_capacity(sections.len());
    for s in sections {
        let collections =
            CollectionRow::filter(CollectionRow::fields().section_slug().eq(s.slug.clone()))
                .order_by(CollectionRow::fields().title().asc())
                .exec(&mut state.db)
                .await
                .context("fetch collections in section")?;
        out.push(SectionWithCollections {
            id: s.id,
            slug: s.slug.clone(),
            title: s.title.clone(),
            nested: s.nested,
            collections: collections
                .into_iter()
                .map(|c| CollectionSummary {
                    id: c.id,
                    title: c.title,
                    section_slug: c.section_slug,
                    section_title: s.title.clone(),
                    nested: s.nested,
                })
                .collect(),
        });
    }
    Ok(out)
}

#[server]
pub async fn inspect_unclassified_folder(rel_path: String) -> Result<FolderInspection, AppError> {
    let root = resolve_unclassified_root()?;
    let root_canon = tokio::fs::canonicalize(&root)
        .await
        .context("canonicalize unclassified root")?;
    inspect_one(&root, &root_canon, &rel_path).await
}

#[server]
pub async fn inspect_unclassified_folders(
    rel_paths: Vec<String>,
) -> Result<FoldersInspection, AppError> {
    if rel_paths.is_empty() {
        return Err(AppError::public("No folders selected"));
    }
    let root = resolve_unclassified_root()?;
    let root_canon = tokio::fs::canonicalize(&root)
        .await
        .context("canonicalize unclassified root")?;

    let mut folders = Vec::with_capacity(rel_paths.len());
    for rp in &rel_paths {
        folders.push(inspect_one(&root, &root_canon, rp).await?);
    }

    let total_files = folders.iter().map(|f| f.file_count).sum();
    let total_size = folders.iter().map(|f| f.total_size).sum();
    let any_has_subdirs = folders.iter().any(|f| f.has_subdirs);

    let common_kind = {
        let mut iter = folders.iter().filter_map(|f| f.dominant_kind);
        let first = iter.next();
        match first {
            Some(k) if folders.iter().all(|f| f.dominant_kind == Some(k)) => Some(k),
            _ => None,
        }
    };

    Ok(FoldersInspection {
        folders,
        total_files,
        total_size,
        any_has_subdirs,
        common_kind,
    })
}

#[cfg(feature = "ssr")]
fn resolve_unclassified_root() -> Result<std::path::PathBuf, AppError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    let root = state
        .config
        .storage
        .unclassified_dir
        .clone()
        .ok_or_else(|| AppError::public("Unclassified media folder is not configured"))?;
    // Sync disjointness check (uses cached canonical media root).
    let root_canon = std::fs::canonicalize(&root)
        .map_err(|e| AppError::public(format!("Cannot open folder: {e}")))?;
    ensure_roots_disjoint(&root_canon, &state.config.storage.media_root)?;
    Ok(root)
}

#[cfg(feature = "ssr")]
async fn inspect_one(
    root: &std::path::Path,
    root_canon: &std::path::Path,
    rel_path: &str,
) -> Result<FolderInspection, AppError> {
    let clean = sanitize_rel(rel_path)?;
    if clean.is_empty() {
        return Err(AppError::public("Invalid folder"));
    }
    let abs_canon = tokio::fs::canonicalize(root.join(&clean))
        .await
        .map_err(|e| AppError::public(format!("Folder not found: {e}")))?;
    if !abs_canon.starts_with(root_canon) {
        return Err(AppError::public("Invalid path"));
    }
    if !tokio::fs::metadata(&abs_canon)
        .await
        .context("stat folder")?
        .is_dir()
    {
        return Err(AppError::public("Not a folder"));
    }

    let name = abs_canon
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Folder".to_string());

    let subdir_names = read_immediate_subdirs(&abs_canon).await?;
    let has_subdirs = !subdir_names.is_empty();
    let files = collect_files(root_canon, &abs_canon, has_subdirs).await?;

    Ok(FolderInspection {
        rel_path: clean,
        name,
        has_subdirs,
        subdir_names,
        file_count: files.len(),
        total_size: files.iter().map(|f| f.size).sum(),
        dominant_kind: dominant_kind(&files),
    })
}

// ══════════════════════════════════════════════════════════════════════════════
//  Poll, cancel
// ══════════════════════════════════════════════════════════════════════════════

#[server]
pub async fn poll_batch(job_ids: Vec<String>) -> Result<Vec<ConversionStatus>, AppError> {
    use crate::app::server::AppState;
    use crate::app::server::convert::JobPhase;

    let state: AppState = expect_context();
    let jobs = state.jobs.read().await;
    let mut out = Vec::with_capacity(job_ids.len());
    for id in &job_ids {
        out.push(match jobs.get(id) {
            Some(job) => match &job.phase {
                JobPhase::Writing => ConversionStatus::Writing,
                JobPhase::Moving {
                    current_file,
                    progress,
                } => ConversionStatus::Moving {
                    current_file: current_file.clone(),
                    progress: *progress,
                },
                JobPhase::Converting {
                    conversion_index,
                    conversion_count,
                    current_file,
                    progress,
                } => ConversionStatus::Converting {
                    conversion_index: *conversion_index,
                    conversion_count: *conversion_count,
                    current_file: current_file.clone(),
                    progress: *progress,
                },
                JobPhase::Finalizing => ConversionStatus::Finalizing,
                JobPhase::Done => ConversionStatus::Done,
                JobPhase::Cancelled => ConversionStatus::Cancelled,
                JobPhase::Failed(e) => ConversionStatus::Failed(e.clone()),
            },
            None => ConversionStatus::Done,
        });
    }
    Ok(out)
}

#[server]
pub async fn cancel_classify(job_ids: Vec<String>) -> Result<(), AppError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    let jobs = state.jobs.read().await;
    for id in &job_ids {
        if let Some(job) = jobs.get(id) {
            job.cancel.cancel();
        }
    }
    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════════
//  Classify files
// ══════════════════════════════════════════════════════════════════════════════

#[server]
pub async fn classify_unclassified_files(
    files: Vec<ClassifyInput>,
    collection_id: Uuid,
    season_number: Option<i64>,
) -> Result<Vec<FileJobHandle>, AppError> {
    use crate::app::model::{CollectionRow, MediaKind, SectionRow};
    use crate::app::server::AppState;

    if files.is_empty() {
        return Err(AppError::public("No files selected"));
    }

    let state: AppState = expect_context();
    let root = state
        .config
        .storage
        .unclassified_dir
        .clone()
        .ok_or_else(|| AppError::public("Unclassified media folder is not configured"))?;
    let root_canon = tokio::fs::canonicalize(&root)
        .await
        .context("canonicalize unclassified root")?;
    ensure_roots_disjoint(&root_canon, &state.config.storage.media_root)?;

    let mut upload_files = Vec::with_capacity(files.len());
    for input in &files {
        upload_files.push(resolve_upload_file(&root, &root_canon, input).await?);
    }

    let paths: Vec<std::path::PathBuf> = upload_files.iter().map(|f| f.temp_path.clone()).collect();
    let _lock = acquire_classify_locks(paths)?;

    let mut db = state.db.clone();
    let collection = CollectionRow::get_by_id(&mut db, &collection_id)
        .await
        .context("load collection")?;
    let section = SectionRow::filter(
        SectionRow::fields()
            .slug()
            .eq(collection.section_slug.clone()),
    )
    .first()
    .exec(&mut db)
    .await
    .context("load section")?
    .ok_or_else(|| AppError::not_found("section"))?;
    let kind = MediaKind::try_from(section.media_kind.as_str())
        .map_err(|e| AppError::internal("section has invalid media_kind", e))?;

    let season_number = if section.nested && kind == MediaKind::Video && season_number.is_none() {
        Some(1)
    } else {
        season_number
    };

    let mut prepared = Vec::with_capacity(upload_files.len());
    for (input, file) in files.into_iter().zip(upload_files) {
        prepared.push(prepare_file_job(
            file,
            collection_id,
            &collection.section_slug,
            &collection.title,
            season_number,
            kind,
            input.rel_path,
        )?);
    }

    let handles = register_and_spawn(
        &state,
        prepared,
        &collection.section_slug,
        Some(collection_id),
    )
    .await;

    let job_ids: Vec<String> = handles.iter().map(|h| h.job_id.clone()).collect();
    spawn_lock_holder(state.clone(), job_ids, _lock);

    Ok(handles)
}

// ══════════════════════════════════════════════════════════════════════════════
//  Classify folders (single and bulk)
// ══════════════════════════════════════════════════════════════════════════════

#[server]
pub async fn classify_unclassified_folder(
    rel_path: String,
    destination: FolderDestination,
) -> Result<Vec<FileJobHandle>, AppError> {
    classify_unclassified_folders(vec![rel_path], destination).await
}

#[server]
pub async fn classify_unclassified_folders(
    rel_paths: Vec<String>,
    destination: FolderDestination,
) -> Result<Vec<FileJobHandle>, AppError> {
    use crate::app::model::{MediaKind, SectionRow};
    use crate::app::server::{AppState, db};

    if rel_paths.is_empty() {
        return Err(AppError::public("No folders selected"));
    }

    let state: AppState = expect_context();
    let root = state
        .config
        .storage
        .unclassified_dir
        .clone()
        .ok_or_else(|| AppError::public("Unclassified media folder is not configured"))?;
    let root_canon = tokio::fs::canonicalize(&root)
        .await
        .context("canonicalize unclassified root")?;
    ensure_roots_disjoint(&root_canon, &state.config.storage.media_root)?;

    // Canonicalize every folder and lock the whole set at once.
    let mut abs_canons: Vec<std::path::PathBuf> = Vec::with_capacity(rel_paths.len());
    for rp in &rel_paths {
        let clean = sanitize_rel(rp)?;
        if clean.is_empty() {
            return Err(AppError::public("Invalid folder"));
        }
        let abs_canon = tokio::fs::canonicalize(root.join(&clean))
            .await
            .map_err(|e| AppError::public(format!("Folder not found: {e}")))?;
        if !abs_canon.starts_with(&root_canon) {
            return Err(AppError::public("Invalid path"));
        }
        if !tokio::fs::metadata(&abs_canon)
            .await
            .context("stat folder")?
            .is_dir()
        {
            return Err(AppError::public("Not a folder"));
        }
        abs_canons.push(abs_canon);
    }
    let _lock = acquire_classify_locks(abs_canons.clone())?;

    let mut db = state.db.clone();

    // ── Resolve destination kind / nested against all folders ────────────────
    enum Dest {
        Existing {
            slug: String,
            kind: MediaKind,
            nested: bool,
        },
        New {
            title: String,
            kind: MediaKind,
            nested: bool,
        },
    }

    let any_has_subdirs = {
        let mut any = false;
        for abs in &abs_canons {
            if !read_immediate_subdirs(abs).await?.is_empty() {
                any = true;
                break;
            }
        }
        any
    };

    let dest = match destination.clone() {
        FolderDestination::Existing { section_slug } => {
            let section = SectionRow::filter(SectionRow::fields().slug().eq(section_slug.clone()))
                .first()
                .exec(&mut db)
                .await
                .context("load section")?
                .ok_or_else(|| AppError::not_found("section"))?;
            let kind = MediaKind::try_from(section.media_kind.as_str())
                .map_err(|e| AppError::internal("section has invalid media_kind", e))?;
            if any_has_subdirs && !section.nested {
                return Err(AppError::public(
                    "At least one folder contains subfolders — choose a nested section.",
                ));
            }
            Dest::Existing {
                slug: section_slug,
                kind,
                nested: section.nested,
            }
        }
        FolderDestination::New { section_title } => {
            let title = section_title.trim().to_string();
            if title.is_empty() {
                return Err(AppError::validation("Section title is required"));
            }
            // Determine a common kind from every folder's files.
            let mut common: Option<MediaKind> = None;
            let mut total_files = 0usize;
            for abs in &abs_canons {
                let files = collect_files(&root_canon, abs, any_has_subdirs).await?;
                total_files += files.len();
                let k = dominant_kind(&files).ok_or_else(|| {
                    AppError::public(
                        "At least one folder is empty or mixes video and audio — split it up before classifying.",
                    )
                })?;
                match common {
                    None => common = Some(k),
                    Some(prev) if prev == k => {}
                    _ => {
                        return Err(AppError::public(
                            "Selected folders mix video and audio — classify them separately.",
                        ));
                    }
                }
            }
            if total_files == 0 {
                return Err(AppError::public("Folders contain no media files"));
            }
            Dest::New {
                title,
                kind: common.unwrap(),
                nested: any_has_subdirs,
            }
        }
    };

    let (kind, nested) = match &dest {
        Dest::Existing { kind, nested, .. } => (*kind, *nested),
        Dest::New { kind, nested, .. } => (*kind, *nested),
    };

    // ── Build groups (one collection per subfolder, or one per folder) ───────
    struct Group {
        title: String,
        season: Option<i64>,
        files: Vec<RawFile>,
        cleanup_dir: std::path::PathBuf,
    }

    let mut groups: Vec<Group> = Vec::new();

    for abs in &abs_canons {
        let subdirs = read_immediate_subdirs(abs).await?;
        if !subdirs.is_empty() {
            for sub in &subdirs {
                let sub_abs = abs.join(sub);
                let files = collect_files(&root_canon, &sub_abs, true).await?;
                if files.is_empty() {
                    continue;
                }
                ensure_kind_matches(&files, kind)?;
                groups.push(Group {
                    title: sub.clone(),
                    season: (kind == MediaKind::Video).then_some(1),
                    files,
                    cleanup_dir: sub_abs,
                });
            }
        } else {
            let files = collect_files(&root_canon, abs, false).await?;
            if files.is_empty() {
                continue;
            }
            ensure_kind_matches(&files, kind)?;
            let folder_name = abs
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Folder".to_string());
            groups.push(Group {
                title: folder_name,
                season: (nested && kind == MediaKind::Video).then_some(1),
                files,
                cleanup_dir: abs.clone(),
            });
        }
    }

    if groups.is_empty() {
        return Err(AppError::public(
            "No media files found in the selected folders",
        ));
    }

    // ── Dedup / resume against existing collection titles ────────────────────
    let existing = match &dest {
        Dest::Existing { slug, .. } => existing_collection_titles(&mut db, slug).await?,
        Dest::New { .. } => Default::default(),
    };

    let mut group_collection_ids: Vec<Option<Uuid>> = vec![None; groups.len()];
    let before = groups.len();
    let mut kept: Vec<Group> = Vec::with_capacity(before);
    let mut kept_ids: Vec<Option<Uuid>> = Vec::with_capacity(before);
    for g in groups {
        if existing.contains_key(&g.title) {
            continue;
        }
        kept.push(g);
        kept_ids.push(None);
    }
    let skipped = before - kept.len();
    if kept.is_empty() {
        return Err(AppError::public(format!(
            "All {before} collection{} already exist in the destination section.",
            plural(before),
        )));
    }
    if skipped > 0 {
        leptos::logging::log!(
            "[classify] skipped {skipped} collection{} already classified",
            plural(skipped),
        );
    }
    groups = kept;
    group_collection_ids = kept_ids;

    // ── Create section (New mode) + collections + prepare per-file jobs ──────
    let section_slug = match &dest {
        Dest::Existing { slug, .. } => slug.clone(),
        Dest::New { title, .. } => {
            let slug = generate_unique_slug(&mut db, kind, nested).await?;
            db::insert_section(&mut db, &slug, title, kind.as_str(), nested)
                .await
                .context("insert section")?;
            slug
        }
    };

    let mut prepared: Vec<PreparedJob> = Vec::new();
    let mut cleanup_watchers: Vec<(Vec<String>, std::path::PathBuf)> = Vec::new();

    for (idx, group) in groups.into_iter().enumerate() {
        let collection_id = match group_collection_ids[idx] {
            Some(id) => id,
            None => db::insert_collection(&mut db, &section_slug, &group.title)
                .await
                .context("create collection")?,
        };

        let mut group_job_ids: Vec<String> = Vec::new();
        for raw in group.files {
            let upload = crate::app::server::upload::UploadFile {
                title: file_stem(&raw.filename),
                filename: raw.filename,
                temp_path: raw.abs,
                size: raw.size,
            };
            let p = prepare_file_job(
                upload,
                collection_id,
                &section_slug,
                &group.title,
                group.season,
                kind,
                raw.rel,
            )?;
            group_job_ids.push(p.job_id.clone());
            prepared.push(p);
        }

        cleanup_watchers.push((group_job_ids, group.cleanup_dir));
    }

    let handles = register_and_spawn(&state, prepared, &section_slug, None).await;

    let all_job_ids: Vec<String> = handles.iter().map(|h| h.job_id.clone()).collect();
    spawn_lock_holder(state.clone(), all_job_ids, _lock);

    for (job_ids, dir) in cleanup_watchers {
        spawn_cleanup_watcher(state.clone(), job_ids, dir);
    }

    Ok(handles)
}

// ══════════════════════════════════════════════════════════════════════════════
//  Move items back to incoming
// ══════════════════════════════════════════════════════════════════════════════

#[server]
pub async fn move_items_to_incoming(item_ids: Vec<Uuid>) -> Result<(), AppError> {
    use crate::app::model::{CollectionRow, File, Item};
    use crate::app::server::AppState;

    if item_ids.is_empty() {
        return Ok(());
    }

    let state: AppState = expect_context();
    let incoming = state
        .config
        .storage
        .unclassified_dir
        .clone()
        .ok_or_else(|| AppError::public("Unclassified media folder is not configured"))?;
    tokio::fs::create_dir_all(&incoming)
        .await
        .context("mkdir incoming")?;
    let incoming_canon = tokio::fs::canonicalize(&incoming)
        .await
        .context("canonicalize incoming")?;
    let media_canon = canonical_media_root(&state.config.storage.media_root)?;

    let mut db = state.db.clone();

    // Track collection count deltas so we update each collection once.
    let mut deltas: std::collections::HashMap<Uuid, i64> = std::collections::HashMap::new();

    for item_id in item_ids {
        let item = match Item::get_by_id(&mut db, &item_id).await {
            Ok(i) => i,
            Err(_) => continue,
        };
        let file = match File::get_by_id(&mut db, &item.file_id).await {
            Ok(f) => f,
            Err(_) => continue,
        };

        // Only move files that live under media_root (skip http:// entries).
        let rel = file.relative_path.clone();
        if rel.starts_with("http://") || rel.starts_with("https://") {
            continue;
        }
        let abs = media_canon.join(&rel);
        if !abs.starts_with(media_canon) {
            continue;
        }
        let abs_canon = match tokio::fs::canonicalize(&abs).await {
            Ok(p) => p,
            Err(_) => continue,
        };
        if !abs_canon.starts_with(media_canon) {
            continue;
        }

        let original_name = abs_canon
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "file".to_string());
        let dest = unique_incoming_path(&incoming_canon, &original_name).await;

        // Try rename; fall back to copy + unlink across filesystems.
        if tokio::fs::rename(&abs_canon, &dest).await.is_err() {
            if tokio::fs::copy(&abs_canon, &dest).await.is_err() {
                continue;
            }
            let _ = tokio::fs::remove_file(&abs_canon).await;
        }

        let collection_id = item.collection_id;
        // Delete item, then file if it's no longer referenced.
        let _ = item.delete().exec(&mut db).await;
        let still_used = !Item::filter(Item::fields().file_id().eq(file.id))
            .limit(1)
            .exec(&mut db)
            .await
            .map(|r| r.is_empty())
            .unwrap_or(false);
        if !still_used {
            let _ = file.delete().exec(&mut db).await;
        }
        *deltas.entry(collection_id).or_insert(0) += 1;
    }

    // Update collection item counts.
    for (cid, delta) in deltas {
        if delta == 0 {
            continue;
        }
        if let Ok(mut c) = CollectionRow::get_by_id(&mut db, &cid).await {
            let current = c.items_count;
            let new_count = (current - delta).max(0);
            let _ = c.update().items_count(new_count).exec(&mut db).await;
        }
    }

    Ok(())
}

#[cfg(feature = "ssr")]
async fn unique_incoming_path(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
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
    // Fallback: uuid suffix. Extremely unlikely.
    dir.join(format!("{stem}-{}{ext}", Uuid::new_v4()))
}

// ══════════════════════════════════════════════════════════════════════════════
//  Server-only helpers used by classify paths
// ══════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "ssr")]
async fn resolve_upload_file(
    root: &std::path::Path,
    root_canon: &std::path::Path,
    input: &ClassifyInput,
) -> Result<crate::app::server::upload::UploadFile, AppError> {
    use crate::app::server::upload::UploadFile;

    let clean = sanitize_rel(&input.rel_path)?;
    if clean.is_empty() {
        return Err(AppError::public("Empty file path"));
    }
    let source_canon = tokio::fs::canonicalize(root.join(&clean))
        .await
        .map_err(|e| AppError::public(format!("File not found: {e}")))?;
    if !source_canon.starts_with(root_canon) {
        return Err(AppError::public("Invalid path"));
    }
    let meta = tokio::fs::metadata(&source_canon)
        .await
        .context("stat source file")?;
    if !meta.is_file() {
        return Err(AppError::public("Not a file"));
    }
    let filename = source_canon
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let title = {
        let t = input.title.trim();
        if t.is_empty() {
            file_stem(&filename)
        } else {
            t.to_string()
        }
    };
    Ok(UploadFile {
        filename,
        title,
        temp_path: source_canon,
        size: meta.len(),
    })
}

#[cfg(feature = "ssr")]
fn make_dummy_tempdir() -> Result<tempfile::TempDir, AppError> {
    tempfile::TempDir::new().map_err(|e| AppError::internal("create dummy temp dir", e))
}

#[cfg(feature = "ssr")]
struct PreparedJob {
    name: String,
    job_id: String,
    rel_path: String,
    title: String,
    season_number: Option<i64>,
    payload: crate::app::server::upload::UploadPayload,
    kind: MediaKind,
    cancel: CancellationToken,
}

#[cfg(feature = "ssr")]
fn prepare_file_job(
    file: crate::app::server::upload::UploadFile,
    collection_id: Uuid,
    section_slug: &str,
    collection_title: &str,
    season: Option<i64>,
    kind: MediaKind,
    rel_path: String,
) -> Result<PreparedJob, AppError> {
    use crate::app::server::upload::UploadPayload;

    let name = file.filename.clone();
    let title = file.title.clone();
    Ok(PreparedJob {
        name,
        job_id: Uuid::new_v4().to_string(),
        rel_path,
        title,
        season_number: season,
        payload: UploadPayload {
            section_slug: section_slug.to_string(),
            collection_id,
            title: collection_title.to_string(),
            description: String::new(),
            season_number: season,
            files: vec![file],
            poster: None,
            temp_dir: make_dummy_tempdir()?,
        },
        kind,
        cancel: CancellationToken::new(),
    })
}

#[cfg(feature = "ssr")]
async fn register_and_spawn(
    state: &crate::app::server::AppState,
    prepared: Vec<PreparedJob>,
    section_slug: &str,
    collection_id: Option<Uuid>,
) -> Vec<FileJobHandle> {
    use crate::app::server::convert::{Job, JobPhase};

    let handles: Vec<FileJobHandle> = prepared
        .iter()
        .map(|p| FileJobHandle {
            name: p.name.clone(),
            job_id: p.job_id.clone(),
            section_slug: section_slug.to_string(),
            collection_id,
            rel_path: p.rel_path.clone(),
            title: p.title.clone(),
            season_number: p.season_number,
        })
        .collect();

    {
        let mut jobs = state.jobs.write().await;
        for p in &prepared {
            jobs.insert(
                p.job_id.clone(),
                Job {
                    phase: JobPhase::Writing,
                    started_at: std::time::Instant::now(),
                    cancel: p.cancel.clone(),
                },
            );
        }
    }

    for p in prepared {
        spawn_job_worker(state.clone(), p);
    }

    handles
}

#[cfg(feature = "ssr")]
fn spawn_job_worker(state: crate::app::server::AppState, p: PreparedJob) {
    use crate::app::server::convert::JobPhase;
    use crate::app::server::upload::{process_upload, schedule_job_eviction};

    tokio::spawn(async move {
        let _permit = upload_semaphore().acquire().await.ok();
        let cancel = p.cancel.clone();

        let outcome = tokio::time::timeout(
            Duration::from_secs(JOB_TIMEOUT_SECS),
            process_upload(p.payload, &state, p.kind, Some(&p.job_id), &cancel),
        )
        .await;

        match outcome {
            Ok(Ok(_)) => {}
            Ok(Err(_)) if cancel.is_cancelled() => {
                crate::app::server::convert::job_set_phase(
                    &state.jobs,
                    Some(&p.job_id),
                    JobPhase::Cancelled,
                )
                .await;
                schedule_job_eviction(state.jobs.clone(), p.job_id);
            }
            Ok(Err(e)) => {
                crate::app::server::convert::job_set_phase(
                    &state.jobs,
                    Some(&p.job_id),
                    JobPhase::Failed(e),
                )
                .await;
                schedule_job_eviction(state.jobs.clone(), p.job_id);
            }
            Err(_) => {
                cancel.cancel();
                let e = AppError::public("Timed out").transient();
                crate::app::server::convert::job_set_phase(
                    &state.jobs,
                    Some(&p.job_id),
                    JobPhase::Failed(e),
                )
                .await;
                schedule_job_eviction(state.jobs.clone(), p.job_id);
            }
        }
    });
}

#[cfg(feature = "ssr")]
async fn wait_for_terminal(state: &crate::app::server::AppState, job_ids: &[String]) {
    use crate::app::server::convert::JobPhase;

    let wait = async {
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            let jobs = state.jobs.read().await;
            let all_terminal = job_ids.iter().all(|id| {
                jobs.get(id)
                    .map(|j| {
                        matches!(
                            j.phase,
                            JobPhase::Done | JobPhase::Cancelled | JobPhase::Failed(_)
                        )
                    })
                    .unwrap_or(true)
            });
            drop(jobs);
            if all_terminal {
                return;
            }
        }
    };

    let _ = tokio::time::timeout(Duration::from_secs(WATCHER_DEADLINE_SECS), wait).await;
}

#[cfg(feature = "ssr")]
fn spawn_lock_holder(
    state: crate::app::server::AppState,
    job_ids: Vec<String>,
    lock: ClassifyLocks,
) {
    tokio::spawn(async move {
        let _lock = lock;
        wait_for_terminal(&state, &job_ids).await;
    });
}

#[cfg(feature = "ssr")]
fn spawn_cleanup_watcher(
    state: crate::app::server::AppState,
    job_ids: Vec<String>,
    dir: std::path::PathBuf,
) {
    tokio::spawn(async move {
        wait_for_terminal(&state, &job_ids).await;
        if let Err(e) = tokio::fs::remove_dir(&dir).await
            && e.kind() != std::io::ErrorKind::NotFound
        {
            leptos::logging::warn!("[classify] could not remove {}: {e}", dir.display());
        }
    });
}

#[cfg(feature = "ssr")]
async fn generate_unique_slug(
    db: &mut toasty::Db,
    kind: MediaKind,
    nested: bool,
) -> Result<String, AppError> {
    use crate::app::server::db;
    let prefix = crate::app::sections::prefix_for(kind, nested);
    for _ in 0..4 {
        let candidate = format!("{prefix}-{}", crate::app::sections::short_hash());
        if db::fetch_section_by_slug(db, &candidate)
            .await
            .context("check slug uniqueness")?
            .is_none()
        {
            return Ok(candidate);
        }
    }
    Err(AppError::internal(
        "create section",
        "could not generate a unique section identifier",
    ))
}

// ══════════════════════════════════════════════════════════════════════════════
//  Client-side batch tracker
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Clone)]
struct FileSlot {
    name: String,
    job_id: String,
    section_slug: String,
    collection_id: Option<Uuid>,
    rel_path: String,
    title: String,
    season_number: Option<i64>,
    status: RwSignal<Option<ConversionStatus>>,
}

impl FileSlot {
    fn from_handle(h: FileJobHandle) -> Self {
        Self {
            name: h.name,
            job_id: h.job_id,
            section_slug: h.section_slug,
            collection_id: h.collection_id,
            rel_path: h.rel_path,
            title: h.title,
            season_number: h.season_number,
            status: RwSignal::new(None),
        }
    }
}

#[derive(Clone)]
enum BatchPhase {
    Setup,
    Running,
    Done,
    Cancelled,
    Failed(AppError),
    Partial {
        succeeded: usize,
        failed: usize,
        cancelled: usize,
        first_error: Option<AppError>,
    },
}

#[derive(Clone, Copy)]
pub struct BatchTracker {
    slots: RwSignal<Vec<FileSlot>>,
    running: RwSignal<bool>,
    error: RwSignal<Option<AppError>>,
    retrying: RwSignal<bool>,
}

impl BatchTracker {
    fn new() -> Self {
        Self {
            slots: RwSignal::new(Vec::new()),
            running: RwSignal::new(false),
            error: RwSignal::new(None),
            retrying: RwSignal::new(false),
        }
    }

    fn begin(&self, handles: Vec<FileJobHandle>) {
        let slots: Vec<FileSlot> = handles.into_iter().map(FileSlot::from_handle).collect();
        self.error.set(None);
        self.retrying.set(false);
        self.slots.set(slots);
        self.running.set(true);
        leptos::task::spawn_local(poll_batch_loop(*self));
    }

    fn reset(&self) {
        self.running.set(false);
        self.retrying.set(false);
        self.slots.set(Vec::new());
        self.error.set(None);
    }

    fn phase(&self) -> Signal<BatchPhase> {
        let slots_sig = self.slots;
        let error = self.error;
        let retrying = self.retrying;
        Signal::derive(move || {
            if retrying.get() {
                return BatchPhase::Running;
            }
            if let Some(e) = error.get() {
                return BatchPhase::Failed(e);
            }
            let slots = slots_sig.get();
            if slots.is_empty() {
                return BatchPhase::Setup;
            }
            let statuses: Vec<Option<ConversionStatus>> =
                slots.iter().map(|s| s.status.get()).collect();
            if statuses.iter().any(|s| s.is_none()) {
                return BatchPhase::Running;
            }
            let mut succeeded = 0usize;
            let mut failed = 0usize;
            let mut cancelled = 0usize;
            let mut first_error: Option<AppError> = None;
            let mut all_terminal = true;
            for s in &statuses {
                match s {
                    Some(ConversionStatus::Done) => succeeded += 1,
                    Some(ConversionStatus::Cancelled) => cancelled += 1,
                    Some(ConversionStatus::Failed(e)) => {
                        failed += 1;
                        if first_error.is_none() {
                            first_error = Some(e.clone());
                        }
                    }
                    Some(_) => all_terminal = false,
                    None => all_terminal = false,
                }
            }
            if !all_terminal {
                return BatchPhase::Running;
            }
            match (succeeded, failed, cancelled) {
                (_, 0, 0) => BatchPhase::Done,
                (0, 0, _) => BatchPhase::Cancelled,
                (0, _, 0) => BatchPhase::Failed(
                    first_error.unwrap_or_else(|| AppError::public("Move failed")),
                ),
                _ => BatchPhase::Partial {
                    succeeded,
                    failed,
                    cancelled,
                    first_error,
                },
            }
        })
    }

    fn slots(&self) -> RwSignal<Vec<FileSlot>> {
        self.slots
    }

    fn pending_job_ids(&self) -> Vec<String> {
        self.slots
            .get_untracked()
            .iter()
            .filter(|s| {
                !matches!(
                    s.status.get_untracked(),
                    Some(ConversionStatus::Done)
                        | Some(ConversionStatus::Cancelled)
                        | Some(ConversionStatus::Failed(_))
                )
            })
            .map(|s| s.job_id.clone())
            .collect()
    }

    fn failed_count(&self) -> usize {
        self.slots
            .get_untracked()
            .iter()
            .filter(|s| matches!(s.status.get_untracked(), Some(ConversionStatus::Failed(_))))
            .count()
    }

    /// Re-dispatch every slot currently in `Failed` state. Groups failures by
    /// `(collection_id, season_number)` so each group can go through the
    /// single-destination classify file fn.
    fn retry_failed(&self) {
        if self.retrying.get_untracked() {
            return;
        }

        // Snapshot failed slots grouped by destination.
        let slots = self.slots.get_untracked();
        let mut groups: std::collections::HashMap<(Uuid, Option<i64>, String), Vec<ClassifyInput>> =
            std::collections::HashMap::new();

        for s in &slots {
            if !matches!(s.status.get_untracked(), Some(ConversionStatus::Failed(_))) {
                continue;
            }
            let Some(cid) = s.collection_id else { continue };
            let key = (cid, s.season_number, s.section_slug.clone());
            groups.entry(key).or_default().push(ClassifyInput {
                rel_path: s.rel_path.clone(),
                title: s.title.clone(),
            });
        }

        if groups.is_empty() {
            return;
        }

        self.retrying.set(true);
        self.error.set(None);

        let tracker = *self;
        leptos::task::spawn_local(async move {
            // For each group, call the classify fn and collect the returned
            // handles in order.
            let mut new_handles: Vec<FileJobHandle> = Vec::new();
            let mut failed_to_start: Option<AppError> = None;

            // Sort keys for deterministic ordering (nice for the UI).
            let mut keys: Vec<_> = groups.keys().cloned().collect();
            keys.sort_by_key(|(_, season, _)| season.unwrap_or(-1));

            for key in keys {
                let (cid, season, _) = key.clone();
                let files = groups.remove(&key).unwrap();
                match classify_unclassified_files(files, cid, season).await {
                    Ok(mut h) => new_handles.append(&mut h),
                    Err(e) => {
                        failed_to_start = Some(e);
                        break;
                    }
                }
            }

            // Merge: replace Failed slots with fresh ones from `new_handles`,
            // preserving the order of the original batch.
            tracker.slots.update(|slots| {
                let mut fresh: std::collections::VecDeque<FileJobHandle> =
                    new_handles.into_iter().collect();
                for slot in slots.iter_mut() {
                    let is_failed = matches!(
                        slot.status.get_untracked(),
                        Some(ConversionStatus::Failed(_))
                    );
                    if !is_failed {
                        continue;
                    }
                    if let Some(next) = fresh.pop_front() {
                        *slot = FileSlot::from_handle(next);
                    }
                }
            });
            tracker.retrying.set(false);

            match failed_to_start {
                Some(e) => tracker.error.set(Some(e)),
                None => {
                    tracker.running.set(true);
                    leptos::task::spawn_local(poll_batch_loop(tracker));
                }
            }
        });
    }
}

async fn poll_batch_loop(tracker: BatchTracker) {
    let mut consecutive_errors = 0u32;

    loop {
        if !tracker.running.get_untracked() {
            return;
        }
        let job_ids: Vec<String> = tracker
            .slots
            .get_untracked()
            .iter()
            .map(|s| s.job_id.clone())
            .collect();
        if job_ids.is_empty() {
            return;
        }

        match poll_batch(job_ids.clone()).await {
            Ok(statuses) => {
                consecutive_errors = 0;
                let all_terminal = statuses.iter().all(|s| {
                    matches!(
                        s,
                        ConversionStatus::Done
                            | ConversionStatus::Cancelled
                            | ConversionStatus::Failed(_)
                    )
                });

                tracker.slots.update(|slots| {
                    for (slot, status) in slots.iter_mut().zip(statuses.iter()) {
                        slot.status.set(Some(status.clone()));
                    }
                });

                if all_terminal {
                    tracker.running.set(false);
                    return;
                }
            }
            Err(e) => {
                consecutive_errors += 1;
                if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                    tracker.error.set(Some(e));
                    tracker.running.set(false);
                    return;
                }
            }
        }

        TimeoutFuture::new(POLL_INTERVAL_MS).await;
    }
}

// ══════════════════════════════════════════════════════════════════════════════
//  Job wrappers
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy)]
pub struct ClassifyJob {
    action: Action<(Vec<ClassifyInput>, Uuid, Option<i64>), Result<Vec<FileJobHandle>, AppError>>,
    tracker: BatchTracker,
}

impl ClassifyJob {
    pub fn new() -> Self {
        let tracker = BatchTracker::new();
        let action = Action::new_local(
            |(files, col, season): &(Vec<ClassifyInput>, Uuid, Option<i64>)| {
                classify_unclassified_files(files.clone(), *col, *season)
            },
        );

        Effect::new(move |_| {
            let Some(val) = action.value().get() else {
                return;
            };
            match val {
                Ok(handles) => tracker.begin(handles),
                Err(e) => tracker.error.set(Some(e)),
            }
        });

        Self { action, tracker }
    }

    pub fn dispatch(&self, files: Vec<ClassifyInput>, col: Uuid, season: Option<i64>) {
        self.tracker.reset();
        self.action.dispatch((files, col, season));
    }

    pub fn reset(&self) {
        self.tracker.reset();
    }

    pub fn tracker(&self) -> BatchTracker {
        self.tracker
    }

    pub fn cancel(&self) {
        let ids = self.tracker.pending_job_ids();
        if ids.is_empty() {
            return;
        }
        leptos::task::spawn_local(async move {
            let _ = cancel_classify(ids).await;
        });
    }
}

#[derive(Clone, Copy)]
pub struct FolderClassifyJob {
    action: Action<(Vec<String>, FolderDestination), Result<Vec<FileJobHandle>, AppError>>,
    tracker: BatchTracker,
}

impl FolderClassifyJob {
    pub fn new() -> Self {
        let tracker = BatchTracker::new();
        let action = Action::new_local(|(paths, dest): &(Vec<String>, FolderDestination)| {
            classify_unclassified_folders(paths.clone(), dest.clone())
        });

        Effect::new(move |_| {
            let Some(val) = action.value().get() else {
                return;
            };
            match val {
                Ok(handles) => tracker.begin(handles),
                Err(e) => tracker.error.set(Some(e)),
            }
        });

        Self { action, tracker }
    }

    pub fn dispatch(&self, paths: Vec<String>, dest: FolderDestination) {
        self.tracker.reset();
        self.action.dispatch((paths, dest));
    }

    pub fn reset(&self) {
        self.tracker.reset();
    }

    pub fn tracker(&self) -> BatchTracker {
        self.tracker
    }

    pub fn cancel(&self) {
        let ids = self.tracker.pending_job_ids();
        if ids.is_empty() {
            return;
        }
        leptos::task::spawn_local(async move {
            let _ = cancel_classify(ids).await;
        });
    }
}

// ══════════════════════════════════════════════════════════════════════════════
//  Page
// ══════════════════════════════════════════════════════════════════════════════

pub struct UnclassifiedPage {
    rel_path: Signal<String>,
    refresh: RwSignal<u32>,
    entries: Resource<Result<Vec<UnclassifiedEntry>, AppError>>,
}

#[lazy_route]
impl LazyRoute for UnclassifiedPage {
    fn data() -> Self {
        let query = use_query_map();
        let rel_path: Signal<String> =
            Signal::derive(move || query.with(|q| q.get("path")).unwrap_or_default());
        let refresh = RwSignal::new(0u32);
        let entries: Resource<Result<Vec<UnclassifiedEntry>, AppError>> = Resource::new(
            move || (rel_path.get(), refresh.get()),
            |(p, _)| list_unclassified(p),
        );
        Self {
            entries,
            rel_path,
            refresh,
        }
    }

    fn view(this: Self) -> AnyView {
        let navigate = use_navigate();

        let on_navigate = Callback::new(move |path: String| {
            let url = if path.is_empty() {
                routes::UNCLASSIFIED.to_string()
            } else {
                format!("{}?path={}", routes::UNCLASSIFIED, encode_query_path(&path))
            };
            navigate(&url, Default::default());
        });

        let file_selection = RwSignal::new(Vec::<UnclassifiedEntry>::new());
        let folder_selection = RwSignal::new(Vec::<UnclassifiedEntry>::new());
        let modal_open = RwSignal::new(false);
        let folder_modal_open = RwSignal::new(false);

        let classify = ClassifyJob::new();
        let folder_classify = FolderClassifyJob::new();

        Effect::new(move |_| {
            let _ = this.rel_path.get();
            file_selection.set(Vec::new());
            folder_selection.set(Vec::new());
            folder_modal_open.set(false);
        });

        Effect::new(move |_| {
            if file_selection.get().is_empty() && modal_open.get() {
                modal_open.set(false);
            }
            if folder_selection.get().is_empty() && folder_modal_open.get() {
                folder_modal_open.set(false);
            }
        });

        let on_modal_close = Callback::new(move |_: ()| {
            file_selection.set(Vec::new());
            this.refresh.update(|n| *n = n.wrapping_add(1));
        });
        let on_folder_modal_close = Callback::new(move |_: ()| {
            folder_selection.set(Vec::new());
            this.refresh.update(|n| *n = n.wrapping_add(1));
        });

        let adapter = move |list: Vec<UnclassifiedEntry>| EntryListProps {
            entries: list,
            on_navigate,
            file_selection,
            folder_selection,
            modal_open,
            folder_modal_open,
            classify,
            folder_classify,
            on_modal_close,
            on_folder_modal_close,
        };

        view! {
            <div class="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
                <div class="mb-5">
                    <h1 class="text-2xl font-bold text-white">"Unclassified media"</h1>
                    <p class="text-gray-400 text-sm mt-1">
                        "Browse the incoming folder and classify files or whole folders."
                    </p>
                </div>
                <Breadcrumb rel_path=this.rel_path on_navigate=on_navigate/>
                <ResourceView resource=this.entries view_fn=render_entry_list adapter=adapter/>
            </div>
        }
        .into_any()
    }
}

// ══════════════════════════════════════════════════════════════════════════════
//  Breadcrumb
// ══════════════════════════════════════════════════════════════════════════════

#[component]
fn Breadcrumb(rel_path: Signal<String>, on_navigate: Callback<String>) -> impl IntoView {
    let segments = Memo::new(move |_| {
        let path = rel_path.get();
        let mut out: Vec<(String, String)> = vec![("Incoming".into(), String::new())];
        if !path.is_empty() {
            let mut acc = String::new();
            for part in path.split('/') {
                if !acc.is_empty() {
                    acc.push('/');
                }
                acc.push_str(part);
                out.push((part.to_string(), acc.clone()));
            }
        }
        out
    });

    view! {
        <nav class="flex items-center gap-1 text-sm mb-4 flex-wrap">
            {move || {
                let segs = segments.get();
                let count = segs.len();
                segs.into_iter().enumerate().map(|(i, (label, path))| {
                    let target = path.clone();
                    let is_last = i + 1 == count;
                    let class = if is_last {
                        "px-2 py-1 rounded transition text-cyan-300 font-medium"
                    } else {
                        "px-2 py-1 rounded transition text-gray-300 hover:bg-white/10 hover:text-white"
                    };
                    view! {
                        <div class="flex items-center gap-1">
                            <button type="button"
                                on:click=move |_| on_navigate.run(target.clone())
                                class=class>{label}</button>
                            {(!is_last).then(|| view! { <span class="text-gray-600">"/"</span> })}
                        </div>
                    }
                }).collect_view()
            }}
        </nav>
    }
}

// ══════════════════════════════════════════════════════════════════════════════
//  Entry list
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Clone)]
pub struct EntryListProps {
    entries: Vec<UnclassifiedEntry>,
    on_navigate: Callback<String>,
    file_selection: RwSignal<Vec<UnclassifiedEntry>>,
    folder_selection: RwSignal<Vec<UnclassifiedEntry>>,
    modal_open: RwSignal<bool>,
    folder_modal_open: RwSignal<bool>,
    classify: ClassifyJob,
    folder_classify: FolderClassifyJob,
    on_modal_close: Callback<()>,
    on_folder_modal_close: Callback<()>,
}

fn render_entry_list(props: EntryListProps) -> AnyView {
    let EntryListProps {
        entries,
        on_navigate,
        file_selection,
        folder_selection,
        modal_open,
        folder_modal_open,
        classify,
        folder_classify,
        on_modal_close,
        on_folder_modal_close,
    } = props;

    if entries.is_empty() {
        return view! {
            <div class="py-12 text-center text-gray-500 text-sm \
                        border border-white/10 rounded-xl bg-white/[0.02]">
                "This folder is empty."
            </div>
        }
        .into_any();
    }

    let entries_for_bar = entries.clone();
    let rows = entries
        .into_iter()
        .map(|entry| {
            if entry.is_dir {
                Either::Left(view! {
                    <DirRow
                        entry
                        on_navigate
                        folder_selection
                    />
                })
            } else {
                Either::Right(view! {
                    <FileRow entry file_selection modal_open/>
                })
            }
        })
        .collect_view();

    view! {
        <div>
            <SelectionBar entries=entries_for_bar file_selection folder_selection modal_open folder_modal_open/>
            <div class="border border-white/10 rounded-xl overflow-hidden bg-white/[0.02]">
                <div class="divide-y divide-white/5">{rows}</div>
            </div>
            <ClassifyModal selection=file_selection open=modal_open job=classify on_close=on_modal_close/>
            <FolderClassifyModal
                selection=folder_selection
                open=folder_modal_open
                job=folder_classify
                on_close=on_folder_modal_close
            />
        </div>
    }
    .into_any()
}

// ══════════════════════════════════════════════════════════════════════════════
//  Selection bar
// ══════════════════════════════════════════════════════════════════════════════

#[component]
fn SelectionBar(
    entries: Vec<UnclassifiedEntry>,
    file_selection: RwSignal<Vec<UnclassifiedEntry>>,
    folder_selection: RwSignal<Vec<UnclassifiedEntry>>,
    modal_open: RwSignal<bool>,
    folder_modal_open: RwSignal<bool>,
) -> impl IntoView {
    let total_supported = entries
        .iter()
        .filter(|e| !e.is_dir && kind_for_ext(e.ext.as_deref()).is_some())
        .count();

    let all_files_selected = move || {
        let n = file_selection.get().len();
        total_supported > 0 && n == total_supported
    };

    let on_select_all = {
        let entries = entries.clone();
        Callback::new(move |ev: web_sys::Event| {
            if event_target_checked(&ev) {
                select_all(file_selection, entries.clone());
            } else {
                file_selection.set(Vec::new());
            }
        })
    };

    let file_count = move || file_selection.get().len();
    let folder_count = move || folder_selection.get().len();
    let total_count = move || file_count() + folder_count();

    let on_clear_all = Callback::new(move |_: ()| {
        file_selection.set(Vec::new());
        folder_selection.set(Vec::new());
    });

    let on_classify = Callback::new(move |_: ()| {
        let files = file_count();
        let folders = folder_count();
        // If there's a folder selection, open the folder modal (it handles
        // mixed selections gracefully by only accepting folder paths).
        if folders > 0 && files == 0 {
            folder_modal_open.set(true);
        } else if files > 0 && folders == 0 {
            modal_open.set(true);
        } else if files > 0 && folders > 0 {
            // Mixed — the folder modal takes priority, but the file modal is
            // not opened because we don't support mixed batches.
            folder_modal_open.set(true);
        }
    });

    view! {
        <div class="flex items-center justify-between gap-3 mb-3 px-1 min-h-9">
            <div class="flex items-center gap-3 min-w-0">
                <Show when=move || { total_supported > 0 }>
                    <label class="flex items-center gap-2 cursor-pointer select-none">
                        <input type="checkbox"
                            prop:checked=all_files_selected
                            on:change=move |ev| on_select_all.run(ev)
                            class="accent-cyan-400 w-4 h-4"/>
                        <span class="text-xs text-gray-400">"Select all files"</span>
                    </label>
                </Show>
                {move || (total_count() > 0).then(|| view! {
                    <span class="text-xs text-cyan-300 font-medium">
                        {move || {
                            let f = file_count();
                            let d = folder_count();
                            match (f, d) {
                                (0, d) => format!("{d} folder{}", plural(d)),
                                (f, 0) => format!("{f} file{}", plural(f)),
                                (f, d) => format!("{f} file{}, {d} folder{}", plural(f), plural(d)),
                            }
                        }}
                    </span>
                })}
            </div>
            <div class="flex items-center gap-2 shrink-0">
                {move || (total_count() > 0).then(|| view! {
                    <>
                        <button type="button"
                            on:click=move |_| on_clear_all.run(())
                            class="text-xs px-2 py-1 rounded-lg text-gray-300 hover:bg-white/10 transition">
                            "Clear"
                        </button>
                        <button type="button"
                            on:click=move |_| on_classify.run(())
                            class="text-xs px-3 py-1.5 rounded-lg \
                                   bg-gradient-to-r from-cyan-500 to-blue-500 \
                                   hover:from-cyan-400 hover:to-blue-400 \
                                   text-white font-bold transition">
                            {move || {
                                let f = file_count();
                                let d = folder_count();
                                if d > 0 && f == 0 {
                                    format!("Classify {d} folder{}", plural(d))
                                } else if f > 0 && d == 0 {
                                    format!("Classify {f} file{}", plural(f))
                                } else {
                                    format!("Classify {} item{}", f + d, plural(f + d))
                                }
                            }}
                        </button>
                    </>
                })}
            </div>
        </div>
    }
}

// ══════════════════════════════════════════════════════════════════════════════
//  Icons
// ══════════════════════════════════════════════════════════════════════════════

#[component]
fn FolderIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 24 24"
             fill="none" stroke="currentColor" stroke-width="1.5"
             stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
        </svg>
    }
}

#[component]
fn FileIcon() -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 24 24"
             fill="none" stroke="currentColor" stroke-width="1.5"
             stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
            <path d="M14 3H7a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8z"/>
            <polyline points="14 3 14 8 19 8"/>
        </svg>
    }
}

// ══════════════════════════════════════════════════════════════════════════════
//  Rows
// ══════════════════════════════════════════════════════════════════════════════

#[component]
fn DirRow(
    entry: UnclassifiedEntry,
    on_navigate: Callback<String>,
    folder_selection: RwSignal<Vec<UnclassifiedEntry>>,
) -> impl IntoView {
    let target = entry.rel_path.clone();
    let name = entry.name.clone();
    let key = entry.rel_path.clone();
    let entry_for_toggle = entry.clone();
    let entry_for_single = entry;

    let is_selected =
        Signal::derive(move || folder_selection.get().iter().any(|e| e.rel_path == key));

    let on_toggle = Callback::new(move |ev: web_sys::Event| {
        if event_target_checked(&ev) {
            toggle_folder_selection(folder_selection, entry_for_toggle.clone());
        } else {
            let path = entry_for_toggle.rel_path.clone();
            folder_selection.update(|v| v.retain(|e| e.rel_path != path));
        }
    });

    let on_single_classify = Callback::new(move |_: ()| {
        folder_selection.set(vec![entry_for_single.clone()]);
    });

    let folder_target = target.clone();
    view! {
        <div class="flex items-center gap-2 pr-3 hover:bg-white/5 transition">
            <input type="checkbox"
                prop:checked=move || is_selected.get()
                on:change=move |ev| on_toggle.run(ev)
                class="accent-cyan-400 w-4 h-4 shrink-0 cursor-pointer ml-4"/>
            <button type="button"
                on:click=move |_| on_navigate.run(folder_target.clone())
                class="flex-1 text-left flex items-center gap-3 px-2 py-2.5 min-w-0">
                <span class="text-cyan-400 shrink-0"><FolderIcon/></span>
                <span class="text-white text-sm truncate flex-1">{name}</span>
            </button>
            <button type="button"
                on:click=move |_| on_single_classify.run(())
                class="text-xs shrink-0 px-2 py-1 rounded-md text-cyan-300 hover:text-white \
                       bg-cyan-500/10 hover:bg-cyan-500/25 border border-cyan-400/30 \
                       transition whitespace-nowrap">
                "Classify"
            </button>
            <button type="button"
                on:click=move |_| on_navigate.run(target.clone())
                class="text-gray-500 text-xs shrink-0 hover:text-white transition">
                "Open →"
            </button>
        </div>
    }
}

#[component]
fn FileRow(
    entry: UnclassifiedEntry,
    file_selection: RwSignal<Vec<UnclassifiedEntry>>,
    modal_open: RwSignal<bool>,
) -> impl IntoView {
    let is_supported = kind_for_ext(entry.ext.as_deref()).is_some();
    let name = entry.name.clone();
    let size = entry.size;
    let key = entry.rel_path.clone();

    let is_selected =
        Signal::derive(move || file_selection.get().iter().any(|e| e.rel_path == key));

    let on_change = {
        let entry = entry.clone();
        Callback::new(move |ev: web_sys::Event| {
            if event_target_checked(&ev) {
                toggle_selection(file_selection, entry.clone());
            } else {
                file_selection.update(|v| v.retain(|e| e.rel_path != entry.rel_path));
            }
        })
    };

    let on_single_classify = Callback::new(move |_: ()| {
        file_selection.set(vec![entry.clone()]);
        modal_open.set(true);
    });

    let row_class = if is_supported {
        "flex items-center gap-3 px-4 py-2.5 transition hover:bg-white/5"
    } else {
        "flex items-center gap-3 px-4 py-2.5 opacity-50"
    };

    view! {
        <div class=row_class>
            <input type="checkbox"
                disabled=!is_supported
                prop:checked=move || is_selected.get()
                on:change=move |ev| on_change.run(ev)
                class="accent-cyan-400 w-4 h-4 shrink-0 cursor-pointer disabled:cursor-not-allowed"/>
            <span class="text-gray-400 shrink-0"><FileIcon/></span>
            <span class="text-white text-sm truncate flex-1">{name}</span>
            <span class="text-gray-500 text-xs font-mono shrink-0">{format_size(size)}</span>
            {if is_supported {
                Either::Left(view! {
                    <button type="button"
                        on:click=move |_| on_single_classify.run(())
                        class="text-xs shrink-0 text-cyan-400 hover:text-cyan-300 transition">
                        "Classify →"
                    </button>
                })
            } else {
                Either::Right(view! {
                    <span class="text-xs shrink-0 text-gray-600">"Unsupported"</span>
                })
            }}
        </div>
    }
}

// ══════════════════════════════════════════════════════════════════════════════
//  Shared modal primitives
// ══════════════════════════════════════════════════════════════════════════════

#[component]
fn ModalShell(
    #[prop(into)] title: String,
    subtitle: Signal<String>,
    busy: Signal<bool>,
    on_close: Callback<()>,
    children: Children,
) -> impl IntoView {
    let try_close = Callback::new(move |_: ()| {
        if !busy.get_untracked() {
            on_close.run(());
        }
    });

    view! {
        <div class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm \
                    flex items-center justify-center p-4"
            on:click=move |_| try_close.run(())>
            <div class="bg-[#12121a] border border-white/10 rounded-2xl p-6 w-full max-w-lg \
                        shadow-2xl max-h-[90vh] overflow-auto"
                on:click=|ev| ev.stop_propagation()>
                <div class="flex items-start justify-between gap-3 mb-4">
                    <div class="min-w-0">
                        <h3 class="text-white font-bold text-lg">{title}</h3>
                        <p class="text-gray-400 text-sm truncate mt-0.5">
                            {move || subtitle.get()}
                        </p>
                    </div>
                    <button type="button"
                        on:click=move |_| try_close.run(())
                        disabled=move || busy.get()
                        class="text-gray-400 hover:text-white transition shrink-0 \
                               disabled:opacity-30 disabled:cursor-not-allowed"
                        aria-label="Close">"×"</button>
                </div>
                {children()}
            </div>
        </div>
    }
}

#[component]
fn TitleField(
    #[prop(default = "Title".to_string(), into)] label: String,
    value: RwSignal<String>,
    #[prop(into)] placeholder: String,
) -> impl IntoView {
    view! {
        <div>
            <label class="text-xs text-gray-500 uppercase tracking-wider">{label}</label>
            <input type="text"
                prop:value=move || value.get()
                on:input=move |e| value.set(event_target_value(&e))
                placeholder=placeholder
                class="mt-1 w-full bg-white/10 text-white rounded-lg px-3 py-1.5 text-sm \
                       focus:outline-none focus:ring-1 focus:ring-cyan-400"/>
        </div>
    }
}

#[component]
fn SectionSelect(
    options: Signal<Vec<SectionWithCollections>>,
    value: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div>
            <label class="text-xs text-gray-500 uppercase tracking-wider">"Section"</label>
            <select
                prop:value=move || value.get()
                on:change=move |e| value.set(event_target_value(&e))
                class="mt-1 w-full bg-white/10 text-white rounded-lg px-3 py-1.5 text-sm \
                       focus:outline-none focus:ring-1 focus:ring-cyan-400">
                <option value="">"Choose a section…"</option>
                {move || options.get().into_iter().map(|s| view! {
                    <option value=s.slug.clone()>{s.title.clone()}</option>
                }).collect_view()}
            </select>
        </div>
    }
}

#[component]
fn SeasonField(value: RwSignal<String>) -> impl IntoView {
    view! {
        <div>
            <label class="text-xs text-gray-500 uppercase tracking-wider">"Season"</label>
            <input type="number" min="1" placeholder="1"
                prop:value=move || value.get()
                on:input=move |e| value.set(event_target_value(&e))
                class="mt-1 w-full bg-white/10 text-white rounded-lg px-3 py-1.5 text-sm \
                       focus:outline-none focus:ring-1 focus:ring-cyan-400"/>
        </div>
    }
}

#[component]
fn FormActions(
    on_cancel: Callback<()>,
    on_confirm: Callback<()>,
    confirm_label: Signal<String>,
    disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="flex justify-end gap-2 pt-2">
            <button type="button"
                on:click=move |_| on_cancel.run(())
                class="px-4 py-2 rounded-lg text-gray-300 hover:bg-white/10 transition text-sm">
                "Cancel"
            </button>
            <button type="button"
                on:click=move |_| on_confirm.run(())
                disabled=move || disabled.get()
                class="px-4 py-2 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 \
                       hover:from-cyan-400 hover:to-blue-400 text-white font-bold text-sm \
                       disabled:opacity-50 transition">
                {move || confirm_label.get()}
            </button>
        </div>
    }
}

// ── Per-file progress bar ─────────────────────────────────────────────────────

#[component]
fn FileProgressBar(name: String, status: RwSignal<Option<ConversionStatus>>) -> impl IntoView {
    view! {
        {move || {
            let s = status.get();
            let (label, pct, bar_class, indeterminate, text_class) = match &s {
                None => ("Waiting…".to_string(), 0.0_f32,
                         "bg-gradient-to-r from-cyan-400 to-blue-500", true, "text-gray-300"),
                Some(ConversionStatus::Writing) => ("Starting…".to_string(), 0.0,
                         "bg-gradient-to-r from-cyan-400 to-blue-500", true, "text-gray-300"),
                Some(ConversionStatus::Moving { progress, .. }) => {
                    let p = if progress.is_finite() { *progress } else { 0.0 };
                    (format!("Moving {:.0}%", p * 100.0), p,
                     "bg-gradient-to-r from-cyan-400 to-blue-500", false, "text-gray-300")
                }
                Some(ConversionStatus::Converting { progress, .. }) => {
                    let p = if progress.is_finite() { *progress } else { 0.0 };
                    (format!("Converting {:.0}%", p * 100.0), p,
                     "bg-gradient-to-r from-cyan-400 to-blue-500", false, "text-gray-300")
                }
                Some(ConversionStatus::Finalizing) => ("Saving…".to_string(), 1.0,
                         "bg-gradient-to-r from-cyan-400 to-blue-500", true, "text-gray-300"),
                Some(ConversionStatus::Done) => ("Done".to_string(), 1.0,
                         "bg-gradient-to-r from-emerald-400 to-green-500", false, "text-emerald-300"),
                Some(ConversionStatus::Cancelled) => ("Cancelled".to_string(), 0.0,
                         "bg-gray-500/50", false, "text-gray-500"),
                Some(ConversionStatus::Failed(e)) => (format!("Failed: {}", e.message), 0.0,
                         "bg-red-500/60", false, "text-red-300"),
            };

            let name = name.clone();
            view! {
                <div class="space-y-1">
                    <div class="flex items-center justify-between gap-2 text-[11px]">
                        <span class=format!("truncate {text_class}")>{name}</span>
                        <span class="text-gray-500 shrink-0">{label}</span>
                    </div>
                    <div class="w-full h-1.5 bg-white/10 rounded-full overflow-hidden">
                        {if indeterminate {
                            Either::Left(view! {
                                <div class="h-full w-1/3 bg-gradient-to-r from-cyan-400 to-blue-500 \
                                            rounded-full animate-pulse"></div>
                            })
                        } else {
                            Either::Right(view! {
                                <div class=format!(
                                        "h-full {} rounded-full transition-all duration-300",
                                        bar_class,
                                    )
                                    style=format!("width: {:.0}%", pct * 100.0)
                                ></div>
                            })
                        }}
                    </div>
                </div>
            }
        }}
    }
}

// ── Batch progress panel with cancel ──────────────────────────────────────────

#[component]
fn BatchProgress(
    tracker: BatchTracker,
    on_cancel: Callback<()>,
    cancel_pending: Signal<bool>,
) -> impl IntoView {
    let slots = tracker.slots();
    let total = move || slots.get().len();
    let done = move || {
        slots
            .get()
            .iter()
            .filter(|s| {
                matches!(
                    s.status.get(),
                    Some(ConversionStatus::Done)
                        | Some(ConversionStatus::Cancelled)
                        | Some(ConversionStatus::Failed(_))
                )
            })
            .count()
    };

    view! {
        <div class="space-y-3">
            <div class="flex items-center justify-between text-xs">
                <span class="text-gray-400">
                    {move || format!("{} / {} done", done(), total())}
                </span>
                <button type="button"
                    on:click=move |_| on_cancel.run(())
                    disabled=move || cancel_pending.get()
                    class="text-xs px-2 py-1 rounded-md text-gray-300 \
                           hover:bg-white/10 hover:text-white transition \
                           disabled:opacity-30 disabled:cursor-not-allowed">
                    {move || if cancel_pending.get() { "Cancelling…" } else { "Cancel" }}
                </button>
            </div>
            <div class="space-y-1.5 max-h-72 overflow-auto pr-1">
                {move || slots.get().into_iter().map(|slot| view! {
                    <FileProgressBar name=slot.name status=slot.status/>
                }).collect_view()}
            </div>
        </div>
    }
}

// ── Failed / cancelled file list ──────────────────────────────────────────────

#[component]
fn FailedFilesList(tracker: BatchTracker) -> impl IntoView {
    let slots = tracker.slots();

    view! {
        {move || {
            let failures: Vec<(String, String, bool)> = slots
                .get()
                .into_iter()
                .filter_map(|s| match s.status.get() {
                    Some(ConversionStatus::Failed(e)) => {
                        Some((s.name.clone(), e.message, true))
                    }
                    Some(ConversionStatus::Cancelled) => {
                        Some((s.name.clone(), "cancelled".to_string(), false))
                    }
                    _ => None,
                })
                .collect();

            if failures.is_empty() {
                return ().into_any();
            }

            view! {
                <div class="text-xs bg-white/[0.03] border border-white/10 \
                            rounded-lg p-2 max-h-32 overflow-auto space-y-0.5">
                    {failures.into_iter().map(|(name, msg, is_error)| {
                        let color = if is_error { "text-red-300/80" } else { "text-gray-400" };
                        view! {
                            <div class="truncate">
                                <span class="text-gray-300">{name}</span>
                                <span class=format!(" ml-1 {color}")>" — " {msg}</span>
                            </div>
                        }
                    }).collect_view()}
                </div>
            }
            .into_any()
        }}
    }
}

// ── Final phase (shared) ──────────────────────────────────────────────────────

#[component]
fn PhaseFinal(
    phase: Signal<BatchPhase>,
    tracker: BatchTracker,
    success_heading: Signal<String>,
    success_detail: Signal<String>,
    link: Signal<Option<(String, Option<Uuid>)>>,
    on_close: Callback<()>,
) -> impl IntoView {
    let retrying = tracker.retrying;
    let failed_count = move || tracker.failed_count();

    let on_retry = Callback::new(move |_: ()| {
        tracker.retry_failed();
    });

    view! {
        <div class="space-y-4">
            <Show when=move || matches!(phase.get(), BatchPhase::Done)>
                <div class="bg-green-500/15 border border-green-500/30 rounded-xl p-4 \
                            flex items-start gap-3">
                    <span class="text-green-400 text-xl leading-none shrink-0">"✓"</span>
                    <div class="min-w-0 flex-1">
                        <div class="text-green-300 font-bold">
                            {move || success_heading.get()}
                        </div>
                        <div class="text-green-300/70 text-xs mt-1">
                            {move || success_detail.get()}
                        </div>
                        {move || link.get().map(|(slug, collection_id)| {
                            let href = match collection_id {
                                Some(id) => routes::collection(&slug, id),
                                None => routes::section(&slug),
                            };
                            let label = if collection_id.is_some() {
                                "View collection →"
                            } else {
                                "View section →"
                            };
                            view! {
                                <a href=href
                                    class="inline-block mt-2 text-xs text-green-300 \
                                           hover:text-green-200 underline underline-offset-2">
                                    {label}
                                </a>
                            }
                        })}
                    </div>
                </div>
            </Show>

            <Show when=move || matches!(phase.get(), BatchPhase::Cancelled)>
                <div class="bg-gray-500/15 border border-gray-500/30 rounded-xl p-4 \
                            flex items-start gap-3">
                    <span class="text-gray-400 text-xl leading-none shrink-0">"■"</span>
                    <div class="min-w-0">
                        <div class="text-gray-300 font-bold">"Cancelled"</div>
                        <div class="text-gray-400 text-xs mt-1">"No files were moved."</div>
                    </div>
                </div>
            </Show>

            <Show when=move || matches!(phase.get(), BatchPhase::Partial { .. })>
                {move || {
                    let (succeeded, failed, cancelled, first_error_msg) = match phase.get() {
                        BatchPhase::Partial { succeeded, failed, cancelled, first_error } => {
                            (succeeded, failed, cancelled, first_error.map(|e| e.message))
                        }
                        _ => return ().into_any(),
                    };
                    let total = succeeded + failed + cancelled;
                    let mut parts: Vec<String> = Vec::new();
                    if succeeded > 0 { parts.push(format!("{succeeded} moved")); }
                    if failed > 0 { parts.push(format!("{failed} failed")); }
                    if cancelled > 0 { parts.push(format!("{cancelled} cancelled")); }
                    let summary = format!("{} of {total}: {}", succeeded, parts.join(", "));
                    view! {
                        <div class="bg-amber-500/15 border border-amber-500/30 rounded-xl p-4 \
                                    flex items-start gap-3">
                            <span class="text-amber-400 text-xl leading-none shrink-0">"!"</span>
                            <div class="min-w-0">
                                <div class="text-amber-300 font-bold">{summary}</div>
                                {first_error_msg.map(|msg| view! {
                                    <div class="text-amber-300/70 text-xs mt-1 break-all">
                                        {format!("First error: {msg}")}
                                    </div>
                                })}
                            </div>
                        </div>
                    }
                    .into_any()
                }}
            </Show>

            <Show when=move || matches!(phase.get(), BatchPhase::Failed(_))>
                <ErrorView
                    error={Signal::derive(move || match phase.get() {
                        BatchPhase::Failed(e) => Some(e),
                        _ => None,
                    })}
                    style=ErrorStyle::Banner
                    heading="Move failed"
                />
            </Show>

            <Show when=move || !matches!(phase.get(), BatchPhase::Done)>
                <FailedFilesList tracker/>
            </Show>

            <div class="flex justify-end gap-2">
                <Show when=move || {failed_count() > 0}>
                    <button type="button"
                        on:click=move |_| on_retry.run(())
                        disabled=move || retrying.get()
                        class="px-4 py-2 rounded-lg bg-amber-500/20 hover:bg-amber-500/30 \
                               text-amber-200 font-bold text-sm transition \
                               disabled:opacity-50">
                        {move || {
                            let n = failed_count();
                            if retrying.get() {
                                "Retrying…".to_string()
                            } else {
                                format!("Retry {n} failed")
                            }
                        }}
                    </button>
                </Show>
                <button type="button"
                    on:click=move |_| on_close.run(())
                    class="px-4 py-2 rounded-lg bg-white/10 hover:bg-white/20 \
                           text-white text-sm transition">
                    "Close"
                </button>
            </div>
        </div>
    }
}

// ══════════════════════════════════════════════════════════════════════════════
//  File classify modal
// ══════════════════════════════════════════════════════════════════════════════

#[derive(Clone, Copy)]
struct FileFormState {
    title: RwSignal<String>,
    section_slug: RwSignal<String>,
    collection_id: RwSignal<Option<Uuid>>,
    season: RwSignal<String>,
    creating_new: RwSignal<bool>,
    new_title: RwSignal<String>,
}

impl FileFormState {
    fn new() -> Self {
        Self {
            title: RwSignal::new(String::new()),
            section_slug: RwSignal::new(String::new()),
            collection_id: RwSignal::new(None),
            season: RwSignal::new(String::new()),
            creating_new: RwSignal::new(false),
            new_title: RwSignal::new(String::new()),
        }
    }

    fn reset(&self) {
        self.title.set(String::new());
        self.section_slug.set(String::new());
        self.collection_id.set(None);
        self.season.set(String::new());
        self.creating_new.set(false);
        self.new_title.set(String::new());
    }
}

#[component]
fn ClassifyModal(
    selection: RwSignal<Vec<UnclassifiedEntry>>,
    open: RwSignal<bool>,
    job: ClassifyJob,
    on_close: Callback<()>,
) -> impl IntoView {
    let file_kind = Memo::new(move |_| {
        selection
            .get()
            .first()
            .and_then(|e| kind_for_ext(e.ext.as_deref()))
    });

    let sections: Resource<Result<Vec<SectionWithCollections>, AppError>> = Resource::new(
        move || {
            open.get()
                .then(|| file_kind.get().map(|k| k.as_str().to_string()))
                .flatten()
        },
        |k| async move {
            match k {
                Some(s) => fetch_sections_with_collections(s).await,
                None => Ok(Vec::new()),
            }
        },
    );

    let form = FileFormState::new();
    let is_single = Signal::derive(move || selection.get().len() == 1);

    let create_action: Action<(String, String), Result<Uuid, AppError>> =
        Action::new_local(|(slug, t): &(String, String)| {
            create_collection_named(slug.clone(), t.clone())
        });

    Effect::new(move |_| {
        if !open.get() {
            return;
        }
        let sel = selection.get_untracked();
        form.reset();
        if let [single] = sel.as_slice() {
            form.title.set(file_stem(&single.name));
        }
    });

    Effect::new(move |_| {
        if !open.get() {
            job.reset();
        }
    });

    Effect::new(move |_| {
        let _ = form.section_slug.get();
        form.collection_id.set(None);
        form.creating_new.set(false);
        form.new_title.set(String::new());
    });

    Effect::new(move |_| {
        if let Some(Ok(id)) = create_action.value().get() {
            sections.refetch();
            form.collection_id.set(Some(id));
            form.creating_new.set(false);
            form.new_title.set(String::new());
        }
    });

    Effect::new(move |_| {
        if form.creating_new.get()
            && form.new_title.get_untracked().is_empty()
            && let Some(first) = selection.get_untracked().first()
        {
            form.new_title.set(file_stem(&first.name));
        }
    });

    let chosen_section: Signal<Option<SectionWithCollections>> = Signal::derive(move || {
        let slug = form.section_slug.get();
        if slug.is_empty() {
            return None;
        }
        sections
            .get()
            .and_then(|r| r.ok())
            .and_then(|list| list.into_iter().find(|s| s.slug == slug))
    });

    let chosen_is_series = Signal::derive(move || {
        file_kind.get() == Some(MediaKind::Video)
            && chosen_section.get().map(|s| s.nested).unwrap_or(false)
    });

    Effect::new(move |_| {
        if chosen_is_series.get() && form.season.get_untracked().trim().is_empty() {
            form.season.set("1".to_string());
        }
    });

    let tracker = job.tracker();
    let phase = tracker.phase();
    let is_busy = Signal::derive(move || matches!(phase.get(), BatchPhase::Running));

    let show_setup = Signal::derive(move || matches!(phase.get(), BatchPhase::Setup));
    let show_running = Signal::derive(move || matches!(phase.get(), BatchPhase::Running));
    let show_final = Signal::derive(move || {
        matches!(
            phase.get(),
            BatchPhase::Done
                | BatchPhase::Cancelled
                | BatchPhase::Partial { .. }
                | BatchPhase::Failed(_)
        )
    });

    let can_move = Signal::derive(move || form.collection_id.get().is_some());

    let confirm_label = Signal::derive(move || {
        let n = selection.get().len();
        if n <= 1 {
            "Move".to_string()
        } else {
            format!("Move {n} files")
        }
    });

    let subtitle = Signal::derive(move || {
        let sel = selection.get();
        match sel.len() {
            0 => String::new(),
            1 => sel[0].name.clone(),
            n => format!("{n} files"),
        }
    });

    let success_heading = Signal::derive(move || {
        let n = selection.get().len();
        if n <= 1 {
            "Moved successfully".to_string()
        } else {
            format!("Moved {n} files")
        }
    });
    let success_detail = Signal::derive(|| "The items are now in the collection.".to_string());

    let link: Signal<Option<(String, Option<Uuid>)>> = Signal::derive(move || {
        tracker
            .slots()
            .get()
            .first()
            .map(|s| (s.section_slug.clone(), s.collection_id))
    });

    let cancel_pending = RwSignal::new(false);

    let try_close = Callback::new(move |_: ()| {
        if is_busy.get_untracked() {
            return;
        }
        let refresh = matches!(
            phase.get_untracked(),
            BatchPhase::Done | BatchPhase::Partial { .. }
        );
        open.set(false);
        if refresh {
            on_close.run(());
        }
    });

    let on_cancel = Callback::new(move |_: ()| {
        if cancel_pending.get_untracked() {
            return;
        }
        cancel_pending.set(true);
        job.cancel();
    });

    let confirm = Callback::new(move |_: ()| {
        let Some(col) = form.collection_id.get_untracked() else {
            return;
        };
        let sel = selection.get_untracked();
        if sel.is_empty() {
            return;
        }
        cancel_pending.set(false);
        let season = parse_season(&form.season.get_untracked());
        let files = if sel.len() == 1 {
            vec![ClassifyInput {
                rel_path: sel[0].rel_path.clone(),
                title: form.title.get_untracked(),
            }]
        } else {
            sel.into_iter()
                .map(|e| ClassifyInput {
                    title: file_stem(&e.name),
                    rel_path: e.rel_path,
                })
                .collect()
        };
        job.dispatch(files, col, season);
    });

    let submit_create = Callback::new(move |_: ()| {
        let slug = form.section_slug.get_untracked();
        let title = form.new_title.get_untracked();
        let title = title.trim();
        if slug.is_empty() || title.is_empty() {
            return;
        }
        create_action.dispatch((slug, title.to_string()));
    });

    let cancel_create = Callback::new(move |_: ()| {
        form.creating_new.set(false);
        form.new_title.set(String::new());
    });

    view! {
        {move || {
            if !open.get() || selection.get().is_empty() {
                return ().into_any();
            }
            let modal_title = if selection.get().len() == 1 {
                "Classify file"
            } else {
                "Classify files"
            };
            view! {
                <ModalShell
                    title=modal_title
                    subtitle=subtitle
                    busy=is_busy
                    on_close=try_close
                >
                    <Show when=move || show_setup.get()>
                        <div class="space-y-4">
                            <Show when=move || is_single.get()>
                                <TitleField value=form.title placeholder="Item title"/>
                            </Show>
                            <Show when=move || !is_single.get()>
                                <div class="text-xs text-gray-500 bg-white/5 \
                                            border border-white/10 rounded-lg px-3 py-2">
                                    "Each file keeps its own name as the item title."
                                </div>
                            </Show>
                            <SectionSelect
                                options={Signal::derive(move || sections.get().and_then(|r| r.ok()).unwrap_or_default())}
                                value=form.section_slug
                            />
                            <Show when=move || !form.section_slug.get().is_empty()>
                                <CollectionPicker
                                    section=chosen_section
                                    collection_id=form.collection_id
                                    creating_new=form.creating_new
                                    new_title=form.new_title
                                    create_pending=create_action.pending().into()
                                    create_error=action_error(create_action)
                                    on_submit_create=submit_create
                                    on_cancel_create=cancel_create
                                />
                            </Show>
                            <Show when=move || chosen_is_series.get()>
                                <SeasonField value=form.season/>
                            </Show>
                            <FormActions
                                on_cancel=try_close
                                on_confirm=confirm
                                confirm_label=confirm_label
                                disabled=Signal::derive(move || !can_move.get())
                            />
                        </div>
                    </Show>
                    <Show when=move || show_running.get()>
                        <BatchProgress
                            tracker=tracker
                            on_cancel=on_cancel
                            cancel_pending=cancel_pending.into()
                        />
                    </Show>
                    <Show when=move || show_final.get()>
                        <PhaseFinal
                            phase=phase
                            tracker=tracker
                            success_heading=success_heading
                            success_detail=success_detail
                            link=link
                            on_close=try_close
                        />
                    </Show>
                </ModalShell>
            }.into_any()
        }}
    }
}

#[component]
fn CollectionPicker(
    section: Signal<Option<SectionWithCollections>>,
    collection_id: RwSignal<Option<Uuid>>,
    creating_new: RwSignal<bool>,
    new_title: RwSignal<String>,
    create_pending: Signal<bool>,
    create_error: Signal<Option<AppError>>,
    on_submit_create: Callback<()>,
    on_cancel_create: Callback<()>,
) -> impl IntoView {
    view! {
        <div>
            <div class="flex items-center justify-between mb-1">
                <label class="text-xs text-gray-500 uppercase tracking-wider">"Collection"</label>
                <Show when=move || !creating_new.get()>
                    <button type="button"
                        on:click=move |_| creating_new.set(true)
                        class="text-xs px-2 py-0.5 rounded-md text-cyan-300 hover:text-white \
                               bg-cyan-500/10 hover:bg-cyan-500/25 border border-cyan-400/30 \
                               transition whitespace-nowrap">
                        "+ New collection"
                    </button>
                </Show>
            </div>

            <Show when=move || !creating_new.get()>
                {move || match section.get() {
                    None => ().into_any(),
                    Some(s) if s.collections.is_empty() => view! {
                        <div class="text-sm text-gray-500 bg-white/5 border border-white/10 \
                                    rounded-lg px-3 py-2">
                            "No collections yet — click “+ New collection” above."
                        </div>
                    }.into_any(),
                    Some(s) => view! {
                        <select
                            prop:value=move || collection_id.get().map(|u| u.to_string()).unwrap_or_default()
                            on:change=move |e| collection_id.set(Uuid::parse_str(&event_target_value(&e)).ok())
                            class="w-full bg-white/10 text-white rounded-lg px-3 py-1.5 text-sm \
                                   focus:outline-none focus:ring-1 focus:ring-cyan-400">
                            <option value="">"Choose a collection…"</option>
                            {s.collections.into_iter().map(|c| view! {
                                <option value=c.id.to_string()>{c.title.clone()}</option>
                            }).collect_view()}
                        </select>
                    }.into_any(),
                }}
            </Show>

            <Show when=move || creating_new.get()>
                <div class="space-y-2">
                    <div class="flex items-stretch gap-2">
                        <input type="text"
                            prop:value=move || new_title.get()
                            on:input=move |e| new_title.set(event_target_value(&e))
                            on:keydown=move |e| {
                                if e.key() == "Enter" { e.prevent_default(); on_submit_create.run(()); }
                                else if e.key() == "Escape" { on_cancel_create.run(()); }
                            }
                            autofocus=true
                            placeholder="New collection title"
                            class="flex-1 bg-white/10 text-white rounded-lg px-3 py-1.5 text-sm \
                                   focus:outline-none focus:ring-1 focus:ring-cyan-400"/>
                        <button type="button"
                            on:click=move |_| on_submit_create.run(())
                            disabled=move || create_pending.get() || new_title.get().trim().is_empty()
                            class="shrink-0 px-3 rounded-lg bg-cyan-500/30 hover:bg-cyan-500/40 \
                                   text-white text-sm font-medium transition disabled:opacity-50">
                            {move || if create_pending.get() { "Creating…" } else { "Create" }}
                        </button>
                        <button type="button"
                            on:click=move |_| on_cancel_create.run(())
                            disabled=move || create_pending.get()
                            class="shrink-0 px-3 rounded-lg bg-white/5 hover:bg-white/15 \
                                   text-gray-300 text-sm transition disabled:opacity-50">
                            "Cancel"
                        </button>
                    </div>
                    <ErrorView error=create_error/>
                </div>
            </Show>
        </div>
    }
}

// ══════════════════════════════════════════════════════════════════════════════
//  Folder classify modal (single and bulk)
// ══════════════════════════════════════════════════════════════════════════════

#[component]
fn FolderClassifyModal(
    selection: RwSignal<Vec<UnclassifiedEntry>>,
    open: RwSignal<bool>,
    job: FolderClassifyJob,
    on_close: Callback<()>,
) -> impl IntoView {
    // Inspect the current folder selection.
    let inspection: Resource<Result<FoldersInspection, AppError>> = Resource::new(
        move || {
            if !open.get() {
                return None;
            }
            Some(
                selection
                    .get()
                    .iter()
                    .map(|e| e.rel_path.clone())
                    .collect::<Vec<_>>(),
            )
        },
        |paths| async move {
            match paths {
                Some(p) if !p.is_empty() => inspect_unclassified_folders(p).await,
                _ => Err(AppError::public("No folder selected")),
            }
        },
    );

    let common_kind = Memo::new(move |_| {
        inspection
            .get()
            .and_then(|r| r.ok())
            .and_then(|i| i.common_kind)
    });

    let sections: Resource<Result<Vec<SectionWithCollections>, AppError>> = Resource::new(
        move || {
            if !open.get() {
                return None;
            }
            let kind = common_kind.get().map(|k| k.as_str().to_string())?;
            Some((selection.get().first()?.rel_path.clone(), kind))
        },
        |input| async move {
            match input {
                Some((_, kind)) => fetch_sections_with_collections(kind).await,
                None => Ok(Vec::new()),
            }
        },
    );

    let mode = RwSignal::new(DestinationMode::New);
    let section_slug = RwSignal::new(String::new());
    let section_title = RwSignal::new(String::new());

    Effect::new(move |_| {
        let _ = open.get();
        mode.set(DestinationMode::New);
        section_slug.set(String::new());
        section_title.set(String::new());
    });

    Effect::new(move |_| {
        if let Some(Ok(insp)) = inspection.get()
            && section_title.get_untracked().trim().is_empty()
            && let Some(first) = insp.folders.first()
        {
            section_title.set(first.name.clone());
        }
    });

    Effect::new(move |_| {
        if !open.get() {
            job.reset();
        }
    });

    let existing_options: Signal<Vec<SectionWithCollections>> = Signal::derive(move || {
        let nested_only = inspection
            .get()
            .and_then(|r| r.ok())
            .map(|i| i.any_has_subdirs)
            .unwrap_or(false);
        sections
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
            .into_iter()
            .filter(|s| !nested_only || s.nested)
            .collect()
    });

    let has_existing = Signal::derive(move || !existing_options.get().is_empty());

    Effect::new(move |_| {
        if inspection.get().is_some() && !has_existing.get() {
            mode.set(DestinationMode::New);
        }
    });

    let tracker = job.tracker();
    let phase = tracker.phase();
    let is_busy = Signal::derive(move || matches!(phase.get(), BatchPhase::Running));

    let show_setup = Signal::derive(move || matches!(phase.get(), BatchPhase::Setup));
    let show_running = Signal::derive(move || matches!(phase.get(), BatchPhase::Running));
    let show_final = Signal::derive(move || {
        matches!(
            phase.get(),
            BatchPhase::Done
                | BatchPhase::Cancelled
                | BatchPhase::Partial { .. }
                | BatchPhase::Failed(_)
        )
    });

    let subtitle = Signal::derive(move || {
        let sel = selection.get();
        match sel.len() {
            0 => String::new(),
            1 => sel[0].name.clone(),
            n => format!("{n} folders"),
        }
    });

    let can_confirm = Signal::derive(move || {
        if !matches!(inspection.get(), Some(Ok(ref i)) if i.common_kind.is_some()) {
            return false;
        }
        match mode.get() {
            DestinationMode::Existing => !section_slug.get().is_empty(),
            DestinationMode::New => !section_title.get().trim().is_empty(),
        }
    });

    let confirm_label = Signal::derive(move || {
        let n = selection.get().len();
        if n <= 1 {
            "Classify folder".to_string()
        } else {
            format!("Classify {n} folders")
        }
    });

    let success_heading = Signal::derive(|| "Folders classified".to_string());
    let success_detail = Signal::derive(move || {
        let files = tracker.slots().get().len();
        format!("{files} file{} moved.", plural(files))
    });

    let link: Signal<Option<(String, Option<Uuid>)>> = Signal::derive(move || {
        tracker
            .slots()
            .get()
            .first()
            .map(|s| (s.section_slug.clone(), s.collection_id))
    });

    let cancel_pending = RwSignal::new(false);

    let try_close = Callback::new(move |_: ()| {
        if is_busy.get_untracked() {
            return;
        }
        let refresh = matches!(
            phase.get_untracked(),
            BatchPhase::Done | BatchPhase::Partial { .. }
        );
        open.set(false);
        if refresh {
            on_close.run(());
        }
    });

    let on_cancel = Callback::new(move |_: ()| {
        if cancel_pending.get_untracked() {
            return;
        }
        cancel_pending.set(true);
        job.cancel();
    });

    let confirm = Callback::new(move |_: ()| {
        let sel = selection.get_untracked();
        if sel.is_empty() {
            return;
        }
        let paths: Vec<String> = sel.iter().map(|e| e.rel_path.clone()).collect();
        let dest = match mode.get_untracked() {
            DestinationMode::Existing => {
                let slug = section_slug.get_untracked();
                if slug.is_empty() {
                    return;
                }
                FolderDestination::Existing { section_slug: slug }
            }
            DestinationMode::New => {
                let t = section_title.get_untracked();
                let t = t.trim();
                if t.is_empty() {
                    return;
                }
                FolderDestination::New {
                    section_title: t.to_string(),
                }
            }
        };
        cancel_pending.set(false);
        job.dispatch(paths, dest);
    });

    view! {
        {move || {
            if !open.get() || selection.get().is_empty() {
                return ().into_any();
            }
            let modal_title = if selection.get().len() == 1 {
                "Classify folder"
            } else {
                "Classify folders"
            };
            view! {
                <ModalShell
                    title=modal_title
                    subtitle=subtitle
                    busy=is_busy
                    on_close=try_close
                >
                    <Show when=move || show_setup.get()>
                        <FolderSetup
                            inspection=inspection
                            existing_options=existing_options
                            has_existing=has_existing
                            mode
                            section_slug
                            section_title
                            can_confirm=can_confirm
                            confirm_label=confirm_label
                            on_cancel=try_close
                            on_confirm=confirm
                        />
                    </Show>
                    <Show when=move || show_running.get()>
                        <BatchProgress
                            tracker=tracker
                            on_cancel=on_cancel
                            cancel_pending=cancel_pending.into()
                        />
                    </Show>
                    <Show when=move || show_final.get()>
                        <PhaseFinal
                            phase=phase
                            tracker=tracker
                            success_heading=success_heading
                            success_detail=success_detail
                            link=link
                            on_close=try_close
                        />
                    </Show>
                </ModalShell>
            }.into_any()
        }}
    }
}

#[component]
fn FolderSetup(
    inspection: Resource<Result<FoldersInspection, AppError>>,
    existing_options: Signal<Vec<SectionWithCollections>>,
    has_existing: Signal<bool>,
    mode: RwSignal<DestinationMode>,
    section_slug: RwSignal<String>,
    section_title: RwSignal<String>,
    can_confirm: Signal<bool>,
    confirm_label: Signal<String>,
    on_cancel: Callback<()>,
    on_confirm: Callback<()>,
) -> impl IntoView {
    view! {
        {move || match inspection.get() {
            None => view! {
                <div class="py-8 text-center text-gray-500 text-sm">"Inspecting folders…"</div>
            }.into_any(),
            Some(Err(e)) => view! {
                <div class="py-4">
                    <ErrorView
                        error=Signal::derive(move || Some(e.clone()))
                        style=ErrorStyle::Banner
                        heading="Cannot classify"/>
                </div>
            }.into_any(),
            Some(Ok(insp)) if insp.common_kind.is_none() => view! {
                <div class="py-4 space-y-3">
                    <ErrorView
                        error=Signal::derive(|| Some(AppError::public(
                            "The selected folders are empty or mix video and audio. \
                             Select folders of the same kind and try again."
                        )))
                        style=ErrorStyle::Banner
                        heading="Cannot classify"/>
                    <div class="flex justify-end">
                        <button type="button"
                            on:click=move |_| on_cancel.run(())
                            class="px-4 py-2 rounded-lg bg-white/10 hover:bg-white/20 \
                                   text-white text-sm transition">
                            "Close"
                        </button>
                    </div>
                </div>
            }.into_any(),
            Some(Ok(insp)) => {
                let kind = insp.common_kind.unwrap();
                let folder_count = insp.folders.len();
                let total_files = insp.total_files;
                let any_has_subdirs = insp.any_has_subdirs;
                let total_size = insp.total_size;
                let kind_label = kind.label().to_string();

                // Build a preview of collection names.
                let mut collections: Vec<String> = Vec::new();
                for f in &insp.folders {
                    if f.has_subdirs {
                        collections.extend(f.subdir_names.iter().cloned());
                    } else {
                        collections.push(f.name.clone());
                    }
                }
                let collection_count = collections.len();
                let collections_preview = StoredValue::new(collections);

                let summary = format!(
                    "{} folder{} · {} file{} · {} · {} · {}",
                    folder_count,
                    plural(folder_count),
                    total_files,
                    plural(total_files),
                    format_size(total_size),
                    kind_label,
                    if any_has_subdirs { "nested" } else { "non-nested" },
                );

                view! {
                    <div class="space-y-4">
                        <div class="text-xs text-gray-400 bg-white/5 border border-white/10 \
                                    rounded-lg px-3 py-2">
                            <div class="text-white text-sm font-medium mb-0.5">{summary}</div>
                            <div>{if any_has_subdirs {
                                "Each subfolder becomes its own collection. Only series / group sections are valid destinations."
                            } else {
                                "Each folder becomes its own collection."
                            }}</div>
                        </div>

                        <DestinationToggle mode has_existing/>

                        <Show when=move || mode.get() == DestinationMode::New>
                            <div class="space-y-3">
                                <TitleField label="Section title" value=section_title placeholder="Section title"/>
                                <Show when=move || {collection_count > 1 || any_has_subdirs}>
                                    <div class="text-xs text-gray-400 bg-white/5 \
                                                border border-white/10 rounded-lg px-3 py-2 \
                                                max-h-40 overflow-auto">
                                        <div class="text-gray-500 uppercase tracking-wider mb-1">
                                            {move || format!(
                                                "{collection_count} new collection{}",
                                                plural(collection_count),
                                            )}
                                        </div>
                                        <ul class="space-y-0.5">
                                            {move || collections_preview.with_value(|v| {
                                                v.iter().map(|name| view! {
                                                    <li class="truncate text-white">"• " {name.clone()}</li>
                                                }).collect_view()
                                            })}
                                        </ul>
                                    </div>
                                </Show>
                            </div>
                        </Show>

                        <Show when=move || mode.get() == DestinationMode::Existing>
                            <SectionSelect options=existing_options value=section_slug/>
                        </Show>

                        <FormActions
                            on_cancel=on_cancel
                            on_confirm=on_confirm
                            confirm_label=confirm_label
                            disabled=Signal::derive(move || !can_confirm.get())
                        />
                    </div>
                }.into_any()
            }
        }}
    }
}

#[component]
fn DestinationToggle(mode: RwSignal<DestinationMode>, has_existing: Signal<bool>) -> impl IntoView {
    let btn = move |this: DestinationMode, label: &'static str| {
        let class = move || {
            let active = mode.get() == this;
            if active {
                "text-xs px-3 py-1.5 rounded-md transition bg-cyan-500/25 text-white"
            } else {
                "text-xs px-3 py-1.5 rounded-md transition text-gray-400 hover:text-white"
            }
        };
        view! {
            <button type="button" on:click=move |_| mode.set(this) class=class>{label}</button>
        }
    };

    view! {
        <div>
            <div class="text-xs text-gray-500 uppercase tracking-wider mb-1">"Destination"</div>
            <div class="inline-flex rounded-lg bg-white/5 border border-white/10 p-0.5">
                {btn(DestinationMode::New, "New section")}
                <Show when=move || has_existing.get()>
                    {btn(DestinationMode::Existing, "Existing section")}
                </Show>
            </div>
        </div>
    }
}
