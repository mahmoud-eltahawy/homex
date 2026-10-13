//! Unclassified media browser.
//!
//! Exposes the optional incoming folder (`HOMEX_UNCLASSIFIED_DIR`) as a
//! navigateable tree. Files, batches of files, and whole folders can be
//! classified into the library.
//!
//! Folder rules:
//!   * A folder that contains subfolders may go into an existing *nested*
//!     section (series / audio group), or into a brand-new nested section.
//!     Each immediate subfolder becomes its own collection.
//!   * A folder that contains only files may go into any existing section
//!     (of the matching kind) as a new collection, or into a brand-new
//!     non-nested section.
//!
//! The current folder is encoded in the URL as `?path=<relative>`, so browser
//! back/forward navigates the tree naturally. The URL is the single source of
//! truth: `rel_path` is a `Signal` derived from the query map.
//!
//! Progress is reported through the shared `state.jobs` map, exactly like the
//! normal upload flow.

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
use crate::app::upload_api::{ConversionStatus, poll_conversion};
use crate::app::upload_job::UploadProgress;

#[cfg(feature = "ssr")]
use crate::app::error::ErrorContext;

const POLL_INTERVAL_MS: u32 = 700;
const MAX_CONSECUTIVE_ERRORS: u32 = 10;

// ─── Wire types ───────────────────────────────────────────────────────────────

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

/// Where the folder should go.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FolderDestination {
    /// Add to an existing section. For files-only folders, a fresh collection
    /// is created; for folders with subfolders, the section must be nested
    /// and each subfolder becomes a collection.
    Existing { section_slug: String },
    /// Create a brand-new section named `section_title`. Non-nested for
    /// files-only folders; nested for folders with subfolders.
    New { section_title: String },
}

// ─── Client-side media-kind inference ─────────────────────────────────────────

const VIDEO_EXTS: &[&str] = &[
    "mp4", "mkv", "mov", "webm", "avi", "m4v", "wmv", "flv", "ts", "mpg", "mpeg", "3gp", "ogv",
];
const AUDIO_EXTS: &[&str] = &[
    "mp3", "m4a", "aac", "wav", "ogg", "oga", "opus", "flac", "wma", "aiff", "aif", "alac", "ape",
];

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

#[cfg(feature = "ssr")]
fn extension_of(name: &str) -> Option<String> {
    name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase())
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

// ─── Selection helpers ────────────────────────────────────────────────────────

fn select_all(selection: RwSignal<Vec<UnclassifiedEntry>>, entries: Vec<UnclassifiedEntry>) {
    selection.update(|v| {
        let target_kind = v
            .first()
            .and_then(|e| kind_for_ext(e.ext.as_deref()))
            .or_else(|| entries.iter().find_map(|e| kind_for_ext(e.ext.as_deref())));
        v.clear();
        for e in entries {
            let k = kind_for_ext(e.ext.as_deref());
            if k.is_none() {
                continue;
            }
            if let Some(tk) = target_kind
                && k != Some(tk)
            {
                continue;
            }
            v.push(e);
        }
    });
}

// ─── Path safety (server-only) ────────────────────────────────────────────────

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

// ─── Server-side folder scanning helpers ──────────────────────────────────────

#[cfg(feature = "ssr")]
#[derive(Debug, Clone)]
struct RawFile {
    abs: std::path::PathBuf,
    filename: String,
    size: u64,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Clone)]
struct WorkItem {
    title: String,
    season: Option<i64>,
    files: Vec<RawFile>,
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
        let ft = match entry.file_type().await {
            Ok(t) => t,
            Err(_) => continue,
        };
        if ft.is_symlink() {
            continue;
        }
        if ft.is_dir() {
            names.push(name);
        }
    }
    names.sort_by_key(|s| s.to_lowercase());
    Ok(names)
}

#[cfg(feature = "ssr")]
async fn collect_files_immediate(abs: &std::path::Path) -> Result<Vec<RawFile>, AppError> {
    let mut rd = tokio::fs::read_dir(abs)
        .await
        .map_err(|e| AppError::public(format!("Cannot read folder: {e}")))?;
    let mut out = Vec::new();
    while let Some(entry) = rd.next_entry().await.context("read dir entry")? {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let ft = match entry.file_type().await {
            Ok(t) => t,
            Err(_) => continue,
        };
        if ft.is_symlink() || !ft.is_file() {
            continue;
        }
        let size = entry.metadata().await.map(|m| m.len()).unwrap_or(0);
        out.push(RawFile {
            abs: entry.path(),
            filename: name,
            size,
        });
    }
    out.sort_by_key(|f| f.filename.to_lowercase());
    Ok(out)
}

#[cfg(feature = "ssr")]
async fn collect_files_recursive(abs: &std::path::Path) -> Result<Vec<RawFile>, AppError> {
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
                stack.push(entry.path());
            } else if ft.is_file() {
                let size = entry.metadata().await.map(|m| m.len()).unwrap_or(0);
                out.push(RawFile {
                    abs: entry.path(),
                    filename: name,
                    size,
                });
            }
        }
    }
    out.sort_by_key(|f| f.filename.to_lowercase());
    Ok(out)
}

/// The media kind shared by every file in `files`, or `None` if there are
/// none or the set mixes video and audio.
#[cfg(feature = "ssr")]
fn dominant_kind(files: &[RawFile]) -> Option<MediaKind> {
    let first = files
        .first()
        .and_then(|f| kind_for_ext(extension_of(&f.filename).as_deref()))?;
    for f in &files[1..] {
        if kind_for_ext(extension_of(&f.filename).as_deref()) != Some(first) {
            return None;
        }
    }
    Some(first)
}

// ─── Server functions ─────────────────────────────────────────────────────────

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

    let clean = sanitize_rel(&rel_path)?;
    let abs = if clean.is_empty() {
        root.clone()
    } else {
        root.join(&clean)
    };

    let root_canon = tokio::fs::canonicalize(&root)
        .await
        .context("canonicalize unclassified root")?;
    let abs_canon = tokio::fs::canonicalize(&abs)
        .await
        .map_err(|e| AppError::public(format!("Cannot open folder: {e}")))?;

    if !abs_canon.starts_with(&root_canon) {
        return Err(AppError::public("Invalid path"));
    }

    let mut rd = tokio::fs::read_dir(&abs_canon)
        .await
        .map_err(|e| AppError::public(format!("Cannot read folder: {e}")))?;

    let mut entries: Vec<UnclassifiedEntry> = Vec::new();
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
        let size = if is_dir { 0 } else { meta.len() };
        let ext = if is_dir { None } else { extension_of(&name) };
        let child_rel = if clean.is_empty() {
            name.clone()
        } else {
            format!("{clean}/{name}")
        };
        entries.push(UnclassifiedEntry {
            name,
            rel_path: child_rel,
            is_dir,
            size,
            ext,
        });
    }

    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
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

    let mut out: Vec<SectionWithCollections> = Vec::with_capacity(sections.len());
    for s in sections {
        let section_title = s.title.clone();
        let section_nested = s.nested;

        let collections =
            CollectionRow::filter(CollectionRow::fields().section_slug().eq(s.slug.clone()))
                .order_by(CollectionRow::fields().title().asc())
                .exec(&mut state.db)
                .await
                .context("fetch collections in section")?;

        out.push(SectionWithCollections {
            id: s.id,
            slug: s.slug,
            title: s.title,
            nested: s.nested,
            collections: collections
                .into_iter()
                .map(|c| CollectionSummary {
                    id: c.id,
                    title: c.title,
                    section_slug: c.section_slug,
                    section_title: section_title.clone(),
                    nested: section_nested,
                })
                .collect(),
        });
    }

    Ok(out)
}

#[server]
pub async fn inspect_unclassified_folder(rel_path: String) -> Result<FolderInspection, AppError> {
    use crate::app::server::AppState;

    let state: AppState = expect_context();
    let root = state
        .config
        .storage
        .unclassified_dir
        .clone()
        .ok_or_else(|| AppError::public("Unclassified media folder is not configured"))?;

    let clean = sanitize_rel(&rel_path)?;
    if clean.is_empty() {
        return Err(AppError::public("Invalid folder"));
    }
    let abs = root.join(&clean);
    let root_canon = tokio::fs::canonicalize(&root)
        .await
        .context("canonicalize unclassified root")?;
    let abs_canon = tokio::fs::canonicalize(&abs)
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

    let name = abs_canon
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Folder".to_string());

    let subdir_names = read_immediate_subdirs(&abs_canon).await?;
    let has_subdirs = !subdir_names.is_empty();

    let files = if has_subdirs {
        collect_files_recursive(&abs_canon).await?
    } else {
        collect_files_immediate(&abs_canon).await?
    };

    let file_count = files.len();
    let total_size = files.iter().map(|f| f.size).sum();

    Ok(FolderInspection {
        rel_path: clean,
        name,
        has_subdirs,
        subdir_names,
        file_count,
        total_size,
        dominant_kind: dominant_kind(&files),
    })
}

#[server]
pub async fn classify_unclassified_files(
    files: Vec<ClassifyInput>,
    collection_id: Uuid,
    season_number: Option<i64>,
) -> Result<String, AppError> {
    use crate::app::model::{CollectionRow, MediaKind, SectionRow};
    use crate::app::server::AppState;
    use crate::app::server::convert::{Job, JobPhase};
    use crate::app::server::upload::{
        UploadFile, UploadPayload, process_upload, schedule_job_eviction,
    };

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

    let mut upload_files: Vec<UploadFile> = Vec::with_capacity(files.len());
    for input in &files {
        let clean = sanitize_rel(&input.rel_path)?;
        if clean.is_empty() {
            return Err(AppError::public("Empty file path"));
        }
        let source_abs = root.join(&clean);
        let source_canon = tokio::fs::canonicalize(&source_abs)
            .await
            .map_err(|e| AppError::public(format!("File not found: {e}")))?;
        if !source_canon.starts_with(&root_canon) {
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
        let fallback_title = file_stem(&filename);
        let display_title = {
            let t = input.title.trim();
            if t.is_empty() {
                fallback_title
            } else {
                t.to_string()
            }
        };
        upload_files.push(UploadFile {
            filename,
            title: display_title,
            temp_path: source_canon,
            size: meta.len(),
        });
    }

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

    let is_video_series = section.nested && kind == MediaKind::Video;
    let season_number = if is_video_series && season_number.is_none() {
        Some(1)
    } else {
        season_number
    };

    let temp_dir =
        tempfile::TempDir::new().map_err(|e| AppError::internal("create dummy temp dir", e))?;

    let payload = UploadPayload {
        section_slug: collection.section_slug.clone(),
        collection_id,
        title: collection.title.clone(),
        description: String::new(),
        season_number,
        files: upload_files,
        poster: None,
        temp_dir,
    };

    let job_id = Uuid::new_v4().to_string();
    {
        let mut jobs = state.jobs.write().await;
        jobs.insert(
            job_id.clone(),
            Job {
                phase: JobPhase::Writing,
                started_at: std::time::Instant::now(),
            },
        );
    }

    let state_bg = state.clone();
    let job_id_bg = job_id.clone();

    tokio::spawn(async move {
        if let Err(e) = process_upload(payload, &state_bg, kind, Some(&job_id_bg)).await {
            crate::app::server::convert::job_set_phase(
                &state_bg.jobs,
                Some(&job_id_bg),
                JobPhase::Failed(e),
            )
            .await;
            schedule_job_eviction(state_bg.jobs.clone(), job_id_bg);
        }
    });

    Ok(job_id)
}

/// Kick off a folder classification job. Returns the job id for polling.
#[server]
pub async fn classify_unclassified_folder(
    rel_path: String,
    destination: FolderDestination,
) -> Result<String, AppError> {
    use crate::app::model::{MediaKind, SectionRow};
    use crate::app::server::convert::{Job, JobPhase};
    use crate::app::server::upload::schedule_job_eviction;
    use crate::app::server::{AppState, db};

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
    let clean = sanitize_rel(&rel_path)?;
    if clean.is_empty() {
        return Err(AppError::public("Invalid folder"));
    }
    let abs = root.join(&clean);
    let abs_canon = tokio::fs::canonicalize(&abs)
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

    let mut db = state.db.clone();

    let subdir_names = read_immediate_subdirs(&abs_canon).await?;
    let has_subdirs = !subdir_names.is_empty();

    // Resolve / create the destination section.
    let (section_slug, section_kind, section_nested) = match destination {
        FolderDestination::Existing { ref section_slug } => {
            let section = SectionRow::filter(SectionRow::fields().slug().eq(section_slug.clone()))
                .first()
                .exec(&mut db)
                .await
                .context("load section")?
                .ok_or_else(|| AppError::not_found("section"))?;
            let kind = MediaKind::try_from(section.media_kind.as_str())
                .map_err(|e| AppError::internal("section has invalid media_kind", e))?;
            if has_subdirs && !section.nested {
                return Err(AppError::public(
                    "This folder contains subfolders — choose a nested section.",
                ));
            }
            (section_slug.to_string(), kind, section.nested)
        }
        FolderDestination::New { ref section_title } => {
            let title = section_title.trim().to_string();
            if title.is_empty() {
                return Err(AppError::validation("Section title is required"));
            }

            // Determine kind from the folder's files.
            let files_for_kind = if has_subdirs {
                collect_files_recursive(&abs_canon).await?
            } else {
                collect_files_immediate(&abs_canon).await?
            };
            if files_for_kind.is_empty() {
                return Err(AppError::public("Folder contains no media files"));
            }
            let kind = dominant_kind(&files_for_kind).ok_or_else(|| {
                AppError::public("Folder mixes video and audio — split it up before classifying.")
            })?;
            let nested = has_subdirs;

            let prefix = crate::app::sections::prefix_for(kind, nested);
            let mut slug: Option<String> = None;
            for attempt in 0..4 {
                let candidate = format!("{prefix}-{}", crate::app::sections::short_hash());
                if db::fetch_section_by_slug(&mut db, &candidate)
                    .await
                    .context("check slug uniqueness")?
                    .is_none()
                {
                    slug = Some(candidate);
                    break;
                }
                if attempt == 3 {
                    return Err(AppError::internal(
                        "create section",
                        "could not generate a unique section identifier",
                    ));
                }
            }
            let slug = slug.unwrap();
            db::insert_section(&mut db, &slug, &title, kind.as_str(), nested)
                .await
                .context("insert section")?;
            (slug, kind, nested)
        }
    };

    // Build the work items.
    let mut work: Vec<WorkItem> = Vec::new();
    let mut cleanup_dirs: Vec<std::path::PathBuf> = Vec::new();

    if has_subdirs {
        for sub in &subdir_names {
            let sub_abs = abs_canon.join(sub);
            let files = collect_files_recursive(&sub_abs).await?;
            if files.is_empty() {
                continue;
            }
            for f in &files {
                if kind_for_ext(extension_of(&f.filename).as_deref()) != Some(section_kind) {
                    return Err(AppError::public(format!(
                        "File '{}' is not {} media",
                        f.filename,
                        section_kind.as_str()
                    )));
                }
            }
            let season = if section_kind == MediaKind::Video {
                Some(1)
            } else {
                None
            };
            cleanup_dirs.push(sub_abs);
            work.push(WorkItem {
                title: sub.clone(),
                season,
                files,
            });
        }
        if work.is_empty() {
            return Err(AppError::public("No media files found in any subfolder"));
        }
    } else {
        let files = collect_files_immediate(&abs_canon).await?;
        if files.is_empty() {
            return Err(AppError::public("Folder is empty"));
        }
        for f in &files {
            if kind_for_ext(extension_of(&f.filename).as_deref()) != Some(section_kind) {
                return Err(AppError::public(format!(
                    "File '{}' is not {} media",
                    f.filename,
                    section_kind.as_str()
                )));
            }
        }
        let folder_name = abs_canon
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Folder".to_string());
        let title = match &destination {
            FolderDestination::New { section_title } => {
                let t = section_title.trim();
                if t.is_empty() {
                    folder_name.clone()
                } else {
                    t.to_string()
                }
            }
            FolderDestination::Existing { .. } => folder_name,
        };
        let season = if section_nested && section_kind == MediaKind::Video {
            Some(1)
        } else {
            None
        };
        cleanup_dirs.push(abs_canon.clone());
        work.push(WorkItem {
            title,
            season,
            files,
        });
    }

    // Create the job and spawn the worker.
    let job_id = Uuid::new_v4().to_string();
    {
        let mut jobs = state.jobs.write().await;
        jobs.insert(
            job_id.clone(),
            Job {
                phase: JobPhase::Writing,
                started_at: std::time::Instant::now(),
            },
        );
    }

    let state_bg = state.clone();
    let job_id_bg = job_id.clone();
    let section_slug_bg = section_slug.clone();

    tokio::spawn(async move {
        let outcome = run_folder_classify(
            work,
            cleanup_dirs,
            section_slug_bg,
            section_kind,
            state_bg.clone(),
            job_id_bg.clone(),
        )
        .await;

        match outcome {
            Ok(()) => {
                crate::app::server::convert::job_set_phase(
                    &state_bg.jobs,
                    Some(&job_id_bg),
                    JobPhase::Done,
                )
                .await;
            }
            Err(e) => {
                crate::app::server::convert::job_set_phase(
                    &state_bg.jobs,
                    Some(&job_id_bg),
                    JobPhase::Failed(e),
                )
                .await;
            }
        }
        schedule_job_eviction(state_bg.jobs.clone(), job_id_bg);
    });

    Ok(job_id)
}

#[cfg(feature = "ssr")]
async fn run_folder_classify(
    work: Vec<WorkItem>,
    cleanup_dirs: Vec<std::path::PathBuf>,
    section_slug: String,
    section_kind: MediaKind,
    state: crate::app::server::AppState,
    job_id: String,
) -> Result<(), AppError> {
    use crate::app::server::convert::JobPhase;
    use crate::app::server::db;
    use crate::app::server::upload::{
        UploadFile, UploadPayload, insert_files, insert_items_and_poster, stage_files,
    };

    let mut db = state.db.clone();

    for item in work {
        let WorkItem {
            title,
            season,
            files,
        } = item;

        let collection_id = db::insert_collection(&mut db, &section_slug, &title)
            .await
            .context("create collection")?;

        let upload_files: Vec<UploadFile> = files
            .into_iter()
            .map(|f| UploadFile {
                title: file_stem(&f.filename),
                filename: f.filename,
                temp_path: f.abs,
                size: f.size,
            })
            .collect();

        let temp_dir =
            tempfile::TempDir::new().map_err(|e| AppError::internal("create dummy temp dir", e))?;

        let payload = UploadPayload {
            section_slug: section_slug.clone(),
            collection_id,
            title,
            description: String::new(),
            season_number: season,
            files: upload_files,
            poster: None,
            temp_dir,
        };

        crate::app::server::convert::job_set_phase(&state.jobs, Some(&job_id), JobPhase::Writing)
            .await;

        let staged = stage_files(&payload, &state, section_kind, Some(&job_id)).await?;
        let file_rows = insert_files(&mut db, &staged).await?;
        insert_items_and_poster(
            &mut db,
            &payload,
            &file_rows,
            &state.config.storage.data_dir,
        )
        .await?;
    }

    // Best-effort cleanup. `remove_dir` fails on a non-empty dir; that's fine.
    for dir in cleanup_dirs {
        let _ = tokio::fs::remove_dir(&dir).await;
    }

    Ok(())
}

// ─── Client-side job trackers ─────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct ClassifyJob {
    action: Action<(Vec<ClassifyInput>, Uuid, Option<i64>), Result<String, AppError>>,
    pub status: RwSignal<Option<ConversionStatus>>,
    pub result: RwSignal<Option<Result<(), AppError>>>,
    active_job: RwSignal<Option<String>>,
}

impl ClassifyJob {
    pub fn new() -> Self {
        let action = Action::new_local(
            |(files, col, season): &(Vec<ClassifyInput>, Uuid, Option<i64>)| {
                classify_unclassified_files(files.clone(), *col, *season)
            },
        );
        let status = RwSignal::new(None);
        let result = RwSignal::new(None);
        let active_job = RwSignal::new(None);

        Effect::new(move |_| {
            let Some(val) = action.value().get() else {
                return;
            };
            match val {
                Ok(id) => active_job.set(Some(id)),
                Err(e) => {
                    status.set(Some(ConversionStatus::Failed(e.clone())));
                    result.set(Some(Err(e)));
                }
            }
        });

        Effect::new(move |_| {
            let Some(id) = active_job.get() else { return };
            leptos::task::spawn_local(poll_until_terminal(id, active_job, status, result));
        });

        Self {
            action,
            status,
            result,
            active_job,
        }
    }

    pub fn dispatch(&self, files: Vec<ClassifyInput>, col: Uuid, season: Option<i64>) {
        self.status.set(Some(ConversionStatus::Writing));
        self.result.set(None);
        self.action.dispatch((files, col, season));
    }

    pub fn reset(&self) {
        self.status.set(None);
        self.result.set(None);
        self.active_job.set(None);
    }

    pub fn error(&self) -> Signal<Option<AppError>> {
        let result = self.result;
        Signal::derive(move || result.get().and_then(|r| r.err()))
    }
}

#[derive(Clone, Copy)]
pub struct FolderClassifyJob {
    action: Action<(String, FolderDestination), Result<String, AppError>>,
    pub status: RwSignal<Option<ConversionStatus>>,
    pub result: RwSignal<Option<Result<(), AppError>>>,
    active_job: RwSignal<Option<String>>,
}

impl FolderClassifyJob {
    pub fn new() -> Self {
        let action = Action::new_local(|(path, dest): &(String, FolderDestination)| {
            classify_unclassified_folder(path.clone(), dest.clone())
        });
        let status = RwSignal::new(None);
        let result = RwSignal::new(None);
        let active_job = RwSignal::new(None);

        Effect::new(move |_| {
            let Some(val) = action.value().get() else {
                return;
            };
            match val {
                Ok(id) => active_job.set(Some(id)),
                Err(e) => {
                    status.set(Some(ConversionStatus::Failed(e.clone())));
                    result.set(Some(Err(e)));
                }
            }
        });

        Effect::new(move |_| {
            let Some(id) = active_job.get() else { return };
            leptos::task::spawn_local(poll_until_terminal(id, active_job, status, result));
        });

        Self {
            action,
            status,
            result,
            active_job,
        }
    }

    pub fn dispatch(&self, path: String, dest: FolderDestination) {
        self.status.set(Some(ConversionStatus::Writing));
        self.result.set(None);
        self.action.dispatch((path, dest));
    }

    pub fn reset(&self) {
        self.status.set(None);
        self.result.set(None);
        self.active_job.set(None);
    }

    pub fn error(&self) -> Signal<Option<AppError>> {
        let result = self.result;
        Signal::derive(move || result.get().and_then(|r| r.err()))
    }
}

async fn poll_until_terminal(
    job_id: String,
    active: RwSignal<Option<String>>,
    status: RwSignal<Option<ConversionStatus>>,
    result: RwSignal<Option<Result<(), AppError>>>,
) {
    let mut consecutive_errors = 0u32;

    loop {
        if active.get_untracked().as_deref() != Some(job_id.as_str()) {
            return;
        }

        let polled = poll_conversion(job_id.clone()).await;

        if active.get_untracked().as_deref() != Some(job_id.as_str()) {
            return;
        }

        match polled {
            Ok(s) => {
                consecutive_errors = 0;
                let terminal = matches!(s, ConversionStatus::Done | ConversionStatus::Failed(_));
                if terminal {
                    let outcome: Result<(), AppError> = match &s {
                        ConversionStatus::Done => Ok(()),
                        ConversionStatus::Failed(e) => Err(e.clone()),
                        _ => unreachable!(),
                    };
                    status.set(Some(s));
                    result.set(Some(outcome));
                    active.set(None);
                    return;
                }
                status.set(Some(s));
            }
            Err(_) => {
                consecutive_errors += 1;
                if consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
                    let e = AppError::public("Lost contact with the server").transient();
                    status.set(Some(ConversionStatus::Failed(e.clone())));
                    result.set(Some(Err(e)));
                    active.set(None);
                    return;
                }
            }
        }

        TimeoutFuture::new(POLL_INTERVAL_MS).await;
    }
}

// ─── Page ─────────────────────────────────────────────────────────────────────

pub struct UnclassifiedPage;

#[lazy_route]
impl LazyRoute for UnclassifiedPage {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        let query = use_query_map();
        let navigate = use_navigate();

        let rel_path: Signal<String> =
            Signal::derive(move || query.with(|q| q.get("path")).unwrap_or_default());

        let on_navigate = Callback::new(move |new_path: String| {
            let url = if new_path.is_empty() {
                routes::UNCLASSIFIED.to_string()
            } else {
                format!(
                    "{}?path={}",
                    routes::UNCLASSIFIED,
                    encode_query_path(&new_path)
                )
            };
            navigate(&url, Default::default());
        });

        let refresh = RwSignal::new(0u32);
        let selection: RwSignal<Vec<UnclassifiedEntry>> = RwSignal::new(Vec::new());
        let modal_open = RwSignal::new(false);
        let folder_target: RwSignal<Option<UnclassifiedEntry>> = RwSignal::new(None);

        let entries: Resource<Result<Vec<UnclassifiedEntry>, AppError>> = Resource::new(
            move || (rel_path.get(), refresh.get()),
            |(path, _)| list_unclassified(path),
        );

        let classify = ClassifyJob::new();
        let folder_classify = FolderClassifyJob::new();

        Effect::new(move |_| {
            let _ = rel_path.get();
            selection.set(Vec::new());
            folder_target.set(None);
        });

        Effect::new(move |_| {
            if selection.get().is_empty() && modal_open.get() {
                modal_open.set(false);
            }
        });

        let on_modal_close = Callback::new(move |_: ()| {
            selection.set(Vec::new());
            refresh.update(|n| *n = n.wrapping_add(1));
        });

        let on_folder_modal_close = Callback::new(move |_: ()| {
            folder_target.set(None);
            refresh.update(|n| *n = n.wrapping_add(1));
        });

        let adapter = move |list: Vec<UnclassifiedEntry>| EntryListProps {
            entries: list,
            on_navigate,
            selection,
            modal_open,
            folder_target,
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

                <Breadcrumb rel_path=rel_path on_navigate=on_navigate/>

                <ResourceView
                    resource=entries
                    view_fn=render_entry_list
                    adapter=adapter
                />
            </div>
        }
        .into_any()
    }
}

// ─── Breadcrumb ───────────────────────────────────────────────────────────────

#[component]
fn Breadcrumb(rel_path: Signal<String>, on_navigate: Callback<String>) -> impl IntoView {
    let segments = Memo::new(move |_| {
        let path = rel_path.get();
        let mut out: Vec<(String, String)> = vec![("Incoming".to_string(), String::new())];
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
                segs.into_iter()
                    .enumerate()
                    .map(|(i, (label, path))| {
                        let target = path.clone();
                        let is_last = i + 1 == count;
                        let class = if is_last {
                            "px-2 py-1 rounded transition text-cyan-300 font-medium"
                        } else {
                            "px-2 py-1 rounded transition text-gray-300 hover:bg-white/10 hover:text-white"
                        };
                        view! {
                            <div class="flex items-center gap-1">
                                <button
                                    type="button"
                                    on:click=move |_| on_navigate.run(target.clone())
                                    class=class
                                >
                                    {label}
                                </button>
                                {(!is_last).then(|| view! {
                                    <span class="text-gray-600">"/"</span>
                                })}
                            </div>
                        }
                    })
                    .collect_view()
            }}
        </nav>
    }
}

// ─── Entry list ───────────────────────────────────────────────────────────────

#[derive(Clone)]
pub struct EntryListProps {
    entries: Vec<UnclassifiedEntry>,
    on_navigate: Callback<String>,
    selection: RwSignal<Vec<UnclassifiedEntry>>,
    modal_open: RwSignal<bool>,
    folder_target: RwSignal<Option<UnclassifiedEntry>>,
    classify: ClassifyJob,
    folder_classify: FolderClassifyJob,
    on_modal_close: Callback<()>,
    on_folder_modal_close: Callback<()>,
}

fn render_entry_list(props: EntryListProps) -> AnyView {
    let EntryListProps {
        entries,
        on_navigate,
        selection,
        modal_open,
        folder_target,
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
                        entry=entry
                        on_navigate=on_navigate
                        folder_target=folder_target
                    />
                })
            } else {
                Either::Right(view! {
                    <FileRow
                        entry=entry
                        selection=selection
                        modal_open=modal_open
                    />
                })
            }
        })
        .collect_view();

    view! {
        <div>
            <SelectionBar
                entries=entries_for_bar
                selection=selection
                modal_open=modal_open
            />
            <div class="border border-white/10 rounded-xl overflow-hidden bg-white/[0.02]">
                <div class="divide-y divide-white/5">{rows}</div>
            </div>
            <ClassifyModal
                selection=selection
                open=modal_open
                classify=classify
                on_close=on_modal_close
            />
            <FolderClassifyModal
                folder_target=folder_target
                job=folder_classify
                on_close=on_folder_modal_close
            />
        </div>
    }
    .into_any()
}

// ─── Selection bar ────────────────────────────────────────────────────────────

#[component]
fn SelectionBar(
    entries: Vec<UnclassifiedEntry>,
    selection: RwSignal<Vec<UnclassifiedEntry>>,
    modal_open: RwSignal<bool>,
) -> impl IntoView {
    let total_supported = entries
        .iter()
        .filter(|e| !e.is_dir && kind_for_ext(e.ext.as_deref()).is_some())
        .count();

    let all_selected = {
        let total = total_supported;
        move || {
            let n = selection.get().len();
            total > 0 && n == total
        }
    };

    let on_select_all = {
        let entries = entries.clone();
        Callback::new(move |ev: web_sys::Event| {
            if event_target_checked(&ev) {
                select_all(selection, entries.clone());
            } else {
                selection.set(Vec::new());
            }
        })
    };

    let on_clear = Callback::new(move |_: ()| selection.set(Vec::new()));

    let on_classify = Callback::new(move |_: ()| {
        if !selection.get_untracked().is_empty() {
            modal_open.set(true);
        }
    });

    view! {
        <div class="flex items-center justify-between gap-3 mb-3 px-1 min-h-9">
            <div class="flex items-center gap-3 min-w-0">
                <Show when=move || { total_supported > 0 }>
                    <label class="flex items-center gap-2 cursor-pointer select-none">
                        <input
                            type="checkbox"
                            prop:checked=move || all_selected()
                            on:change=move |ev| on_select_all.run(ev)
                            class="accent-cyan-400 w-4 h-4"
                        />
                        <span class="text-xs text-gray-400">"Select all"</span>
                    </label>
                </Show>
                {move || {
                    let n = selection.get().len();
                    if n == 0 {
                        None
                    } else {
                        Some(view! {
                            <span class="text-xs text-cyan-300 font-medium">
                                {format!("{n} selected")}
                            </span>
                        })
                    }
                }}
            </div>

            <div class="flex items-center gap-2 shrink-0">
                {move || {
                    let n = selection.get().len();
                    if n == 0 {
                        None
                    } else {
                        Some(view! {
                            <>
                                <button
                                    type="button"
                                    on:click=move |_| on_clear.run(())
                                    class="text-xs px-2 py-1 rounded-lg text-gray-300 \
                                           hover:bg-white/10 transition"
                                >
                                    "Clear"
                                </button>
                                <button
                                    type="button"
                                    on:click=move |_| on_classify.run(())
                                    class="text-xs px-3 py-1.5 rounded-lg \
                                           bg-gradient-to-r from-cyan-500 to-blue-500 \
                                           hover:from-cyan-400 hover:to-blue-400 \
                                           text-white font-bold transition"
                                >
                                    {move || format!("Classify {n}")}
                                </button>
                            </>
                        })
                    }
                }}
            </div>
        </div>
    }
}

// ─── Row icons ────────────────────────────────────────────────────────────────

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

// ─── Rows ─────────────────────────────────────────────────────────────────────

#[component]
fn DirRow(
    entry: UnclassifiedEntry,
    on_navigate: Callback<String>,
    folder_target: RwSignal<Option<UnclassifiedEntry>>,
) -> impl IntoView {
    let target = entry.rel_path.clone();
    let name = entry.name.clone();
    let entry_for_classify = entry.clone();

    let on_open = Callback::new(move |_: ()| on_navigate.run(target.clone()));
    let on_classify = Callback::new(move |_: ()| {
        folder_target.set(Some(entry_for_classify.clone()));
    });

    view! {
        <div class="flex items-center gap-2 pr-3 hover:bg-white/5 transition">
            <button
                type="button"
                on:click=move |_| on_open.run(())
                class="flex-1 text-left flex items-center gap-3 px-4 py-2.5 min-w-0"
            >
                <span class="w-4 shrink-0"></span>
                <span class="text-cyan-400 shrink-0"><FolderIcon/></span>
                <span class="text-white text-sm truncate flex-1">{name}</span>
            </button>
            <button
                type="button"
                on:click=move |_| on_classify.run(())
                class="text-xs shrink-0 px-2 py-1 rounded-md \
                       text-cyan-300 hover:text-white \
                       bg-cyan-500/10 hover:bg-cyan-500/25 \
                       border border-cyan-400/30 transition whitespace-nowrap"
            >
                "Classify"
            </button>
            <button
                type="button"
                on:click=move |_| on_open.run(())
                class="text-gray-500 text-xs shrink-0 hover:text-white transition"
            >
                "Open →"
            </button>
        </div>
    }
}

#[component]
fn FileRow(
    entry: UnclassifiedEntry,
    selection: RwSignal<Vec<UnclassifiedEntry>>,
    modal_open: RwSignal<bool>,
) -> impl IntoView {
    let is_supported = kind_for_ext(entry.ext.as_deref()).is_some();
    let name = entry.name.clone();
    let size = entry.size;
    let rel_path_key = entry.rel_path.clone();

    let entry_for_change = entry.clone();
    let entry_for_classify = entry.clone();

    let is_selected =
        Signal::derive(move || selection.get().iter().any(|e| e.rel_path == rel_path_key));

    let on_change = Callback::new(move |ev: web_sys::Event| {
        let checked = event_target_checked(&ev);
        let entry = entry_for_change.clone();
        selection.update(|v| {
            if checked {
                if v.iter().any(|e| e.rel_path == entry.rel_path) {
                    return;
                }
                let new_kind = kind_for_ext(entry.ext.as_deref());
                let existing_kind = v.first().and_then(|e| kind_for_ext(e.ext.as_deref()));
                if existing_kind.is_some() && existing_kind != new_kind {
                    v.clear();
                }
                v.push(entry);
            } else {
                v.retain(|e| e.rel_path != entry.rel_path);
            }
        });
    });

    let on_single_classify = Callback::new(move |_: ()| {
        selection.set(vec![entry_for_classify.clone()]);
        modal_open.set(true);
    });

    let row_class = if is_supported {
        "flex items-center gap-3 px-4 py-2.5 transition hover:bg-white/5"
    } else {
        "flex items-center gap-3 px-4 py-2.5 opacity-50"
    };

    view! {
        <div class=row_class>
            <input
                type="checkbox"
                disabled=!is_supported
                prop:checked=move || is_selected.get()
                on:change=move |ev| on_change.run(ev)
                class="accent-cyan-400 w-4 h-4 shrink-0 cursor-pointer \
                       disabled:cursor-not-allowed"
            />
            <span class="text-gray-400 shrink-0"><FileIcon/></span>
            <span class="text-white text-sm truncate flex-1">{name}</span>
            <span class="text-gray-500 text-xs font-mono shrink-0">
                {format_size(size)}
            </span>
            {if is_supported {
                Either::Left(view! {
                    <button
                        type="button"
                        on:click=move |_| on_single_classify.run(())
                        class="text-xs shrink-0 text-cyan-400 hover:text-cyan-300 \
                               transition"
                    >
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

// ─── File classify modal (unchanged) ──────────────────────────────────────────

#[component]
fn ClassifyModal(
    selection: RwSignal<Vec<UnclassifiedEntry>>,
    open: RwSignal<bool>,
    classify: ClassifyJob,
    on_close: Callback<()>,
) -> impl IntoView {
    let file_kind: Memo<Option<MediaKind>> = Memo::new(move |_| {
        selection
            .get()
            .first()
            .and_then(|e| kind_for_ext(e.ext.as_deref()))
    });

    let sections: Resource<Result<Vec<SectionWithCollections>, AppError>> = Resource::new(
        move || {
            if !open.get() {
                return None;
            }
            file_kind.get().map(|k| k.as_str().to_string())
        },
        |k| async move {
            match k {
                Some(s) => fetch_sections_with_collections(s).await,
                None => Ok(Vec::new()),
            }
        },
    );

    let is_single = Signal::derive(move || selection.get().len() == 1);

    let single_title = RwSignal::new(String::new());
    let chosen_section_slug = RwSignal::new(String::new());
    let chosen_collection_id = RwSignal::new(None::<Uuid>);
    let season = RwSignal::new(String::new());

    let creating_new = RwSignal::new(false);
    let new_collection_title = RwSignal::new(String::new());

    let create_action: Action<(String, String), Result<Uuid, AppError>> =
        Action::new_local(|(slug, t): &(String, String)| {
            create_collection_named(slug.clone(), t.clone())
        });

    Effect::new(move |_| {
        if let Some(Ok(id)) = create_action.value().get() {
            sections.refetch();
            chosen_collection_id.set(Some(id));
            creating_new.set(false);
            new_collection_title.set(String::new());
        }
    });

    Effect::new(move |_| {
        if open.get() {
            let sel = selection.get_untracked();
            if sel.len() == 1 {
                single_title.set(file_stem(&sel[0].name));
            } else {
                single_title.set(String::new());
            }
            chosen_section_slug.set(String::new());
            chosen_collection_id.set(None);
            season.set(String::new());
            creating_new.set(false);
            new_collection_title.set(String::new());
        }
    });

    Effect::new(move |_| {
        if !open.get() {
            classify.reset();
        }
    });

    Effect::new(move |_| {
        let _ = chosen_section_slug.get();
        chosen_collection_id.set(None);
        creating_new.set(false);
        new_collection_title.set(String::new());
    });

    let chosen_section: Signal<Option<SectionWithCollections>> = Signal::derive(move || {
        let slug = chosen_section_slug.get();
        if slug.is_empty() {
            return None;
        }
        sections
            .get()
            .and_then(|r| r.ok())
            .and_then(|list| list.into_iter().find(|s| s.slug == slug))
    });

    let chosen_is_series = Signal::derive(move || {
        if file_kind.get() != Some(MediaKind::Video) {
            return false;
        }
        chosen_section.get().map(|s| s.nested).unwrap_or(false)
    });

    Effect::new(move |_| {
        if chosen_is_series.get() && season.get_untracked().trim().is_empty() {
            season.set("1".to_string());
        }
    });

    let try_close = Callback::new(move |_: ()| {
        let busy =
            classify.status.get_untracked().is_some() && classify.result.get_untracked().is_none();
        if busy {
            return;
        }
        let success = matches!(classify.result.get_untracked(), Some(Ok(())));
        open.set(false);
        if success {
            on_close.run(());
        }
    });

    let confirm = Callback::new(move |_: ()| {
        let sel = selection.get_untracked();
        if sel.is_empty() {
            return;
        }
        let Some(col) = chosen_collection_id.get_untracked() else {
            return;
        };
        let s = season.get_untracked();
        let s = s.trim();
        let season_num: Option<i64> = if s.is_empty() { None } else { s.parse().ok() };

        let files: Vec<ClassifyInput> = if sel.len() == 1 {
            vec![ClassifyInput {
                rel_path: sel[0].rel_path.clone(),
                title: single_title.get_untracked(),
            }]
        } else {
            sel.into_iter()
                .map(|e| ClassifyInput {
                    title: file_stem(&e.name),
                    rel_path: e.rel_path,
                })
                .collect()
        };

        classify.dispatch(files, col, season_num);
    });

    let submit_create = Callback::new(move |_: ()| {
        let slug = chosen_section_slug.get_untracked();
        if slug.is_empty() {
            return;
        }
        let t = new_collection_title.get_untracked();
        let t = t.trim();
        if t.is_empty() {
            return;
        }
        create_action.dispatch((slug, t.to_string()));
    });

    let cancel_create = Callback::new(move |_: ()| {
        creating_new.set(false);
        new_collection_title.set(String::new());
    });

    Effect::new(move |_| {
        if creating_new.get() && new_collection_title.get_untracked().is_empty() {
            if let Some(first) = selection.get_untracked().first() {
                new_collection_title.set(file_stem(&first.name));
            }
        }
    });

    let show_setup = Signal::derive(move || classify.status.get().is_none());
    let show_progress =
        Signal::derive(move || classify.status.get().is_some() && classify.result.get().is_none());
    let show_final = Signal::derive(move || classify.result.get().is_some());
    let is_busy =
        Signal::derive(move || classify.status.get().is_some() && classify.result.get().is_none());

    let status_signal = Signal::derive(move || classify.status.get());
    let error_signal = classify.error();
    let create_error = action_error(create_action);
    let create_pending = create_action.pending();

    let can_move = Signal::derive(move || chosen_collection_id.get().is_some());

    view! {
        {move || {
            if !open.get() {
                return ().into_any();
            }
            let sel = selection.get();
            if sel.is_empty() {
                return ().into_any();
            }

            let count = sel.len();
            let header_line = if count == 1 {
                sel[0].name.clone()
            } else {
                format!("{count} files")
            };

            view! {
                <div
                    class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm \
                           flex items-center justify-center p-4"
                    on:click=move |_| try_close.run(())
                >
                    <div
                        class="bg-[#12121a] border border-white/10 rounded-2xl p-6 \
                               w-full max-w-lg shadow-2xl max-h-[90vh] overflow-auto"
                        on:click=|ev| ev.stop_propagation()
                    >
                        <div class="flex items-start justify-between gap-3 mb-4">
                            <div class="min-w-0">
                                <h3 class="text-white font-bold text-lg">
                                    {if count == 1 { "Classify file" } else { "Classify files" }}
                                </h3>
                                <p class="text-gray-400 text-sm truncate mt-0.5">
                                    {header_line}
                                </p>
                            </div>
                            <button
                                type="button"
                                on:click=move |_| try_close.run(())
                                disabled=move || is_busy.get()
                                class="text-gray-400 hover:text-white transition shrink-0 \
                                       disabled:opacity-30 disabled:cursor-not-allowed"
                                aria-label="Close"
                            >
                                "×"
                            </button>
                        </div>

                        <Show when=move || show_setup.get()>
                            <div class="space-y-4">
                                <Show when=move || is_single.get()>
                                    <div>
                                        <label class="text-xs text-gray-500 \
                                                      uppercase tracking-wider">
                                            "Title"
                                        </label>
                                        <input
                                            type="text"
                                            prop:value=move || single_title.get()
                                            on:input=move |e| single_title.set(event_target_value(&e))
                                            placeholder="Item title"
                                            class="mt-1 w-full bg-white/10 text-white \
                                                   rounded-lg px-3 py-1.5 text-sm \
                                                   focus:outline-none focus:ring-1 \
                                                   focus:ring-cyan-400"
                                        />
                                    </div>
                                </Show>

                                <Show when=move || !is_single.get()>
                                    <div class="text-xs text-gray-500 bg-white/5 \
                                                border border-white/10 rounded-lg \
                                                px-3 py-2">
                                        "Each file keeps its own name as the item title."
                                    </div>
                                </Show>

                                <div>
                                    <label class="text-xs text-gray-500 \
                                                  uppercase tracking-wider">
                                        "Section"
                                    </label>
                                    <select
                                        prop:value=move || chosen_section_slug.get()
                                        on:change=move |e| {
                                            chosen_section_slug.set(event_target_value(&e))
                                        }
                                        class="mt-1 w-full bg-white/10 text-white \
                                               rounded-lg px-3 py-1.5 text-sm \
                                               focus:outline-none focus:ring-1 \
                                               focus:ring-cyan-400"
                                    >
                                        <option value="">"Choose a section…"</option>
                                        {move || {
                                            sections
                                                .get()
                                                .and_then(|r| r.ok())
                                                .unwrap_or_default()
                                                .into_iter()
                                                .map(|s| view! {
                                                    <option value=s.slug.clone()>
                                                        {s.title.clone()}
                                                    </option>
                                                })
                                                .collect_view()
                                        }}
                                    </select>
                                </div>

                                <Show when=move || !chosen_section_slug.get().is_empty()>
                                    <div>
                                        <div class="flex items-center justify-between mb-1">
                                            <label class="text-xs text-gray-500 \
                                                          uppercase tracking-wider">
                                                "Collection"
                                            </label>
                                            <Show when=move || !creating_new.get()>
                                                <button
                                                    type="button"
                                                    on:click=move |_| creating_new.set(true)
                                                    class="text-xs px-2 py-0.5 rounded-md \
                                                           text-cyan-300 hover:text-white \
                                                           bg-cyan-500/10 hover:bg-cyan-500/25 \
                                                           border border-cyan-400/30 \
                                                           transition whitespace-nowrap"
                                                >
                                                    "+ New collection"
                                                </button>
                                            </Show>
                                        </div>

                                        <Show when=move || !creating_new.get()>
                                            {move || {
                                                let Some(s) = chosen_section.get() else {
                                                    return ().into_any();
                                                };
                                                if s.collections.is_empty() {
                                                    return view! {
                                                        <div class="text-sm text-gray-500 \
                                                                    bg-white/5 border border-white/10 \
                                                                    rounded-lg px-3 py-2">
                                                            "No collections yet — click “+ New collection” above."
                                                        </div>
                                                    }
                                                    .into_any();
                                                }
                                                view! {
                                                    <select
                                                        prop:value=move || {
                                                            chosen_collection_id
                                                                .get()
                                                                .map(|u| u.to_string())
                                                                .unwrap_or_default()
                                                        }
                                                        on:change=move |e| {
                                                            let v = event_target_value(&e);
                                                            chosen_collection_id.set(
                                                                Uuid::parse_str(&v).ok(),
                                                            );
                                                        }
                                                        class="w-full bg-white/10 text-white \
                                                               rounded-lg px-3 py-1.5 text-sm \
                                                               focus:outline-none focus:ring-1 \
                                                               focus:ring-cyan-400"
                                                    >
                                                        <option value="">
                                                            "Choose a collection…"
                                                        </option>
                                                        {s.collections
                                                            .into_iter()
                                                            .map(|c| view! {
                                                                <option value=c.id.to_string()>
                                                                    {c.title.clone()}
                                                                </option>
                                                            })
                                                            .collect_view()}
                                                    </select>
                                                }
                                                .into_any()
                                            }}
                                        </Show>

                                        <Show when=move || creating_new.get()>
                                            <div class="space-y-2">
                                                <div class="flex items-stretch gap-2">
                                                    <input
                                                        type="text"
                                                        prop:value=move || new_collection_title.get()
                                                        on:input=move |e| {
                                                            new_collection_title.set(event_target_value(&e))
                                                        }
                                                        on:keydown=move |e| {
                                                            if e.key() == "Enter" {
                                                                e.prevent_default();
                                                                submit_create.run(());
                                                            } else if e.key() == "Escape" {
                                                                cancel_create.run(());
                                                            }
                                                        }
                                                        autofocus=true
                                                        placeholder="New collection title"
                                                        class="flex-1 bg-white/10 text-white \
                                                               rounded-lg px-3 py-1.5 text-sm \
                                                               focus:outline-none focus:ring-1 \
                                                               focus:ring-cyan-400"
                                                    />
                                                    <button
                                                        type="button"
                                                        on:click=move |_| submit_create.run(())
                                                        disabled=move || {
                                                            create_pending.get()
                                                                || new_collection_title
                                                                    .get()
                                                                    .trim()
                                                                    .is_empty()
                                                        }
                                                        class="shrink-0 px-3 rounded-lg \
                                                               bg-cyan-500/30 hover:bg-cyan-500/40 \
                                                               text-white text-sm font-medium \
                                                               transition disabled:opacity-50"
                                                    >
                                                        {move || if create_pending.get() {
                                                            "Creating…"
                                                        } else {
                                                            "Create"
                                                        }}
                                                    </button>
                                                    <button
                                                        type="button"
                                                        on:click=move |_| cancel_create.run(())
                                                        disabled=move || create_pending.get()
                                                        class="shrink-0 px-3 rounded-lg \
                                                               bg-white/5 hover:bg-white/15 \
                                                               text-gray-300 text-sm \
                                                               transition disabled:opacity-50"
                                                    >
                                                        "Cancel"
                                                    </button>
                                                </div>
                                                <ErrorView error=create_error/>
                                            </div>
                                        </Show>
                                    </div>
                                </Show>

                                <Show when=move || chosen_is_series.get()>
                                    <div>
                                        <label class="text-xs text-gray-500 \
                                                      uppercase tracking-wider">
                                            "Season"
                                        </label>
                                        <input
                                            type="number"
                                            min="1"
                                            placeholder="1"
                                            prop:value=move || season.get()
                                            on:input=move |e| season.set(event_target_value(&e))
                                            class="mt-1 w-full bg-white/10 text-white \
                                                   rounded-lg px-3 py-1.5 text-sm \
                                                   focus:outline-none focus:ring-1 \
                                                   focus:ring-cyan-400"
                                        />
                                    </div>
                                </Show>

                                <div class="flex justify-end gap-2 pt-2">
                                    <button
                                        type="button"
                                        on:click=move |_| try_close.run(())
                                        class="px-4 py-2 rounded-lg text-gray-300 \
                                               hover:bg-white/10 transition text-sm"
                                    >
                                        "Cancel"
                                    </button>
                                    <button
                                        type="button"
                                        on:click=move |_| confirm.run(())
                                        disabled=move || !can_move.get()
                                        class="px-4 py-2 rounded-lg \
                                               bg-gradient-to-r from-cyan-500 to-blue-500 \
                                               hover:from-cyan-400 hover:to-blue-400 \
                                               text-white font-bold text-sm \
                                               disabled:opacity-50 transition"
                                    >
                                        {move || {
                                            let n = selection.get().len();
                                            if n <= 1 {
                                                "Move".to_string()
                                            } else {
                                                format!("Move {n} files")
                                            }
                                        }}
                                    </button>
                                </div>
                            </div>
                        </Show>

                        <Show when=move || show_progress.get()>
                            <div class="space-y-3">
                                <UploadProgress status=status_signal/>
                                <div class="text-xs text-gray-500 text-center pt-2">
                                    "Please wait — files are being moved."
                                </div>
                            </div>
                        </Show>

                        <Show when=move || show_final.get()>
                            <div class="space-y-4">
                                <Show when=move || {
                                    matches!(classify.result.get(), Some(Ok(())))
                                }>
                                    <div class="bg-green-500/15 border border-green-500/30 \
                                                rounded-xl p-4 flex items-start gap-3">
                                        <span class="text-green-400 text-xl \
                                                     leading-none shrink-0">
                                            "✓"
                                        </span>
                                        <div class="min-w-0">
                                            <div class="text-green-300 font-bold">
                                                {move || {
                                                    let n = selection.get().len();
                                                    if n <= 1 { "Moved successfully".to_string() }
                                                    else { format!("Moved {n} files") }
                                                }}
                                            </div>
                                            <div class="text-green-300/70 text-xs mt-1">
                                                "The items are now in the collection."
                                            </div>
                                        </div>
                                    </div>
                                </Show>

                                <Show when=move || {
                                    matches!(classify.result.get(), Some(Err(_)))
                                }>
                                    <ErrorView
                                        error=error_signal
                                        style=ErrorStyle::Banner
                                        heading="Move failed"
                                    />
                                </Show>

                                <div class="flex justify-end">
                                    <button
                                        type="button"
                                        on:click=move |_| try_close.run(())
                                        class="px-4 py-2 rounded-lg bg-white/10 \
                                               hover:bg-white/20 text-white text-sm \
                                               transition"
                                    >
                                        "Close"
                                    </button>
                                </div>
                            </div>
                        </Show>
                    </div>
                </div>
            }
            .into_any()
        }}
    }
}

// ─── Folder classify modal ────────────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq)]
enum DestinationMode {
    Existing,
    New,
}

#[component]
fn FolderClassifyModal(
    folder_target: RwSignal<Option<UnclassifiedEntry>>,
    job: FolderClassifyJob,
    on_close: Callback<()>,
) -> impl IntoView {
    // Inspection resource — runs whenever the target folder changes.
    let inspection: Resource<Result<FolderInspection, AppError>> = Resource::new(
        move || folder_target.get().map(|e| e.rel_path),
        |maybe_path| async move {
            match maybe_path {
                Some(path) => inspect_unclassified_folder(path).await,
                None => Err(AppError::public("No folder selected")),
            }
        },
    );

    // Existing sections for the folder's dominant kind.
    let sections: Resource<Result<Vec<SectionWithCollections>, AppError>> = Resource::new(
        move || {
            let kind = inspection
                .get()
                .and_then(|r| r.ok())
                .and_then(|i| i.dominant_kind)
                .map(|k| k.as_str().to_string());
            (folder_target.get().map(|e| e.rel_path), kind)
        },
        |(_path, kind)| async move {
            match kind {
                Some(s) => fetch_sections_with_collections(s).await,
                None => Ok(Vec::new()),
            }
        },
    );

    let mode: RwSignal<DestinationMode> = RwSignal::new(DestinationMode::New);
    let chosen_section_slug = RwSignal::new(String::new());
    let section_title = RwSignal::new(String::new());

    // Reset the picker whenever the target folder changes.
    Effect::new(move |_| {
        let _ = folder_target.get();
        // Default to New section — that's the primary flow for folder classify.
        mode.set(DestinationMode::New);
        chosen_section_slug.set(String::new());
        section_title.set(String::new());
    });

    // Prefill the section title with the folder name once inspection resolves.
    Effect::new(move |_| {
        if let Some(Ok(insp)) = inspection.get()
            && section_title.get_untracked().trim().is_empty()
        {
            section_title.set(insp.name.clone());
        }
    });

    // If inspection succeeds and there are no usable existing sections, force
    // New mode so the user isn't stuck on an empty dropdown.
    Effect::new(move |_| {
        if let Some(Ok(insp)) = inspection.get()
            && let Some(Ok(list)) = sections.get()
        {
            let usable = list.iter().any(|s| !insp.has_subdirs || s.nested);
            if !usable {
                mode.set(DestinationMode::New);
            }
        }
    });

    Effect::new(move |_| {
        if folder_target.get().is_none() {
            job.reset();
        }
    });

    let try_close = Callback::new(move |_: ()| {
        let busy = job.status.get_untracked().is_some() && job.result.get_untracked().is_none();
        if busy {
            return;
        }
        let success = matches!(job.result.get_untracked(), Some(Ok(())));
        folder_target.set(None);
        if success {
            on_close.run(());
        }
    });

    let confirm = Callback::new(move |_: ()| {
        let Some(entry) = folder_target.get_untracked() else {
            return;
        };
        let dest = match mode.get_untracked() {
            DestinationMode::Existing => {
                let slug = chosen_section_slug.get_untracked();
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
        job.dispatch(entry.rel_path.clone(), dest);
    });

    let show_setup = Signal::derive(move || job.status.get().is_none());
    let show_progress =
        Signal::derive(move || job.status.get().is_some() && job.result.get().is_none());
    let show_final = Signal::derive(move || job.result.get().is_some());
    let is_busy = Signal::derive(move || job.status.get().is_some() && job.result.get().is_none());

    let status_signal = Signal::derive(move || job.status.get());
    let error_signal = job.error();

    let can_move = Signal::derive(move || {
        if !matches!(inspection.get(), Some(Ok(i)) if i.dominant_kind.is_some()) {
            return false;
        }
        match mode.get() {
            DestinationMode::Existing => !chosen_section_slug.get().is_empty(),
            DestinationMode::New => !section_title.get().trim().is_empty(),
        }
    });

    view! {
        {move || {
            let Some(entry) = folder_target.get() else {
                return ().into_any();
            };
            let folder_name = entry.name.clone();

            view! {
                <div
                    class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm \
                           flex items-center justify-center p-4"
                    on:click=move |_| try_close.run(())
                >
                    <div
                        class="bg-[#12121a] border border-white/10 rounded-2xl p-6 \
                               w-full max-w-lg shadow-2xl max-h-[90vh] overflow-auto"
                        on:click=|ev| ev.stop_propagation()
                    >
                        <div class="flex items-start justify-between gap-3 mb-4">
                            <div class="min-w-0">
                                <h3 class="text-white font-bold text-lg">
                                    "Classify folder"
                                </h3>
                                <p class="text-gray-400 text-sm truncate mt-0.5">
                                    {folder_name}
                                </p>
                            </div>
                            <button
                                type="button"
                                on:click=move |_| try_close.run(())
                                disabled=move || is_busy.get()
                                class="text-gray-400 hover:text-white transition shrink-0 \
                                       disabled:opacity-30 disabled:cursor-not-allowed"
                                aria-label="Close"
                            >
                                "×"
                            </button>
                        </div>

                        <Show when=move || show_setup.get()>
                            {move || match inspection.get() {
                                None => view! {
                                    <div class="py-8 text-center text-gray-500 text-sm">
                                        "Inspecting folder…"
                                    </div>
                                }
                                .into_any(),
                                Some(Err(e)) => view! {
                                    <div class="py-4">
                                        <ErrorView
                                            error=Signal::derive(move || Some(e.clone()))
                                            style=ErrorStyle::Banner
                                            heading="Cannot classify folder"
                                        />
                                    </div>
                                }
                                .into_any(),
                                Some(Ok(insp)) => {
                                    if insp.dominant_kind.is_none() {
                                        return view! {
                                            <div class="py-4 space-y-3">
                                                <ErrorView
                                                    error=Signal::derive(move || Some(AppError::public(
                                                        "This folder has no media files, or mixes video and audio. Split it up or move it manually."
                                                    )))
                                                    style=ErrorStyle::Banner
                                                    heading="Cannot classify folder"
                                                />
                                                <div class="flex justify-end">
                                                    <button
                                                        type="button"
                                                        on:click=move |_| try_close.run(())
                                                        class="px-4 py-2 rounded-lg bg-white/10 \
                                                               hover:bg-white/20 text-white text-sm transition"
                                                    >
                                                        "Close"
                                                    </button>
                                                </div>
                                            </div>
                                        }
                                        .into_any();
                                    }

                                    let has_subdirs = insp.has_subdirs;
                                    let file_count = insp.file_count;
                                    let total_size = insp.total_size;
                                    let subdir_names = insp.subdir_names.clone();
                                    let n_subdirs = subdir_names.len();
                                    let subdir_names = StoredValue::new(subdir_names);
                                    let kind = insp.dominant_kind.unwrap();
                                    let kind_str = kind.label().to_string();
                                    let nested_hint = if has_subdirs {
                                        "nested"
                                    } else {
                                        "non-nested"
                                    };

                                    let summary_line = if has_subdirs {
                                        format!(
                                            "{n_subdirs} subfolder{} · {file_count} file{} · {} · {} {}",
                                            if n_subdirs == 1 { "" } else { "s" },
                                            if file_count == 1 { "" } else { "s" },
                                            format_size(total_size),
                                            kind_str,
                                            nested_hint,
                                        )
                                    } else {
                                        format!(
                                            "{file_count} file{} · {} · {} {}",
                                            if file_count == 1 { "" } else { "s" },
                                            format_size(total_size),
                                            kind_str,
                                            nested_hint,
                                        )
                                    };

                                    let explanation = if has_subdirs {
                                        "Each subfolder becomes its own collection. Only series / group sections are valid destinations."
                                    } else {
                                        "A new collection is created and every file here becomes an item in it."
                                    };

                                    // Sections the user may pick in Existing mode.
                                    let available_sections: Signal<Vec<SectionWithCollections>> =
                                        Signal::derive(move || {
                                            sections
                                                .get()
                                                .and_then(|r| r.ok())
                                                .unwrap_or_default()
                                                .into_iter()
                                                .filter(|s| !has_subdirs || s.nested)
                                                .collect()
                                        });

                                    let has_usable_existing = {
                                        let list_signal = available_sections;
                                        Signal::derive(move || !list_signal.get().is_empty())
                                    };

                                    view! {
                                        <div class="space-y-4">
                                            <div class="text-xs text-gray-400 bg-white/5 \
                                                        border border-white/10 rounded-lg \
                                                        px-3 py-2">
                                                <div class="text-white text-sm font-medium mb-0.5">
                                                    {summary_line}
                                                </div>
                                                <div>{explanation}</div>
                                            </div>

                                            // ── Destination mode toggle ──
                                            <div>
                                                <div class="text-xs text-gray-500 \
                                                            uppercase tracking-wider mb-1">
                                                    "Destination"
                                                </div>
                                                <div class="inline-flex rounded-lg \
                                                            bg-white/5 border border-white/10 p-0.5">
                                                    <button
                                                        type="button"
                                                        on:click=move |_| mode.set(DestinationMode::New)
                                                        class=move || {
                                                            let active = mode.get() == DestinationMode::New;
                                                            format!(
                                                                "text-xs px-3 py-1.5 rounded-md transition {}",
                                                                if active {
                                                                    "bg-cyan-500/25 text-white"
                                                                } else {
                                                                    "text-gray-400 hover:text-white"
                                                                },
                                                            )
                                                        }
                                                    >
                                                        "New section"
                                                    </button>
                                                    <Show when=move || has_usable_existing.get()>
                                                        <button
                                                            type="button"
                                                            on:click=move |_| mode.set(DestinationMode::Existing)
                                                            class=move || {
                                                                let active = mode.get() == DestinationMode::Existing;
                                                                format!(
                                                                    "text-xs px-3 py-1.5 rounded-md transition {}",
                                                                    if active {
                                                                        "bg-cyan-500/25 text-white"
                                                                    } else {
                                                                        "text-gray-400 hover:text-white"
                                                                    },
                                                                )
                                                            }
                                                        >
                                                            "Existing section"
                                                        </button>
                                                    </Show>
                                                </div>
                                            </div>

                                            // ── New section ──
                                            <Show when=move || mode.get() == DestinationMode::New>
                                                <div>
                                                    <label class="text-xs text-gray-500 \
                                                                  uppercase tracking-wider">
                                                        "Section title"
                                                    </label>
                                                    <input
                                                        type="text"
                                                        prop:value=move || section_title.get()
                                                        on:input=move |e| {
                                                            section_title.set(event_target_value(&e))
                                                        }
                                                        placeholder="Section title"
                                                        class="mt-1 w-full bg-white/10 text-white \
                                                               rounded-lg px-3 py-1.5 text-sm \
                                                               focus:outline-none focus:ring-1 \
                                                               focus:ring-cyan-400"
                                                    />
                                                </div>
                                                <div class="text-xs text-gray-500 \
                                                            bg-white/5 border border-white/10 \
                                                            rounded-lg px-3 py-2">
                                                    {if has_subdirs {
                                                        format!(
                                                            "A new nested section is created. Each subfolder becomes a collection:",
                                                        )
                                                    } else {
                                                        "A new non-nested section is created, containing one collection with all files.".to_string()
                                                    }}
                                                </div>
                                                <Show when=move || has_subdirs>
                                                    <div class="text-xs text-gray-400 bg-white/5 border border-white/10 \
                                                                rounded-lg px-3 py-2 max-h-40 overflow-auto">
                                                        <ul class="space-y-0.5">
                                                    {move || subdir_names
                                                        .read_value()
                                                        .iter()
                                                        .map(|name| {
                                                            let name = name.clone();
                                                            view! { <li class="truncate text-white">"• " {name}</li> }
                                                        })
                                                        .collect_view()}

                                                        </ul>
                                                    </div>
                                                </Show>
                                            </Show>

                                            // ── Existing section ──
                                            <Show when=move || mode.get() == DestinationMode::Existing>
                                                <div>
                                                    <label class="text-xs text-gray-500 \
                                                                  uppercase tracking-wider">
                                                        "Section"
                                                    </label>
                                                    <select
                                                        prop:value=move || chosen_section_slug.get()
                                                        on:change=move |e| {
                                                            chosen_section_slug.set(event_target_value(&e))
                                                        }
                                                        class="mt-1 w-full bg-white/10 text-white \
                                                               rounded-lg px-3 py-1.5 text-sm \
                                                               focus:outline-none focus:ring-1 \
                                                               focus:ring-cyan-400"
                                                    >
                                                        <option value="">"Choose a section…"</option>
                                                        {move || {
                                                            available_sections
                                                                .get()
                                                                .into_iter()
                                                                .map(|s| view! {
                                                                    <option value=s.slug.clone()>
                                                                        {s.title.clone()}
                                                                    </option>
                                                                })
                                                                .collect_view()
                                                        }}
                                                    </select>
                                                </div>
                                            </Show>

                                            <div class="flex justify-end gap-2 pt-2">
                                                <button
                                                    type="button"
                                                    on:click=move |_| try_close.run(())
                                                    class="px-4 py-2 rounded-lg text-gray-300 \
                                                           hover:bg-white/10 transition text-sm"
                                                >
                                                    "Cancel"
                                                </button>
                                                <button
                                                    type="button"
                                                    on:click=move |_| confirm.run(())
                                                    disabled=move || !can_move.get()
                                                    class="px-4 py-2 rounded-lg \
                                                           bg-gradient-to-r from-cyan-500 to-blue-500 \
                                                           hover:from-cyan-400 hover:to-blue-400 \
                                                           text-white font-bold text-sm \
                                                           disabled:opacity-50 transition"
                                                >
                                                    {if has_subdirs {
                                                        format!("Classify folder ({n_subdirs})")
                                                    } else {
                                                        "Classify folder".to_string()
                                                    }}
                                                </button>
                                            </div>
                                        </div>
                                    }
                                    .into_any()
                                }
                            }}
                        </Show>

                        <Show when=move || show_progress.get()>
                            <div class="space-y-3">
                                <UploadProgress status=status_signal/>
                                <div class="text-xs text-gray-500 text-center pt-2">
                                    "Please wait — files are being moved."
                                </div>
                            </div>
                        </Show>

                        <Show when=move || show_final.get()>
                            <div class="space-y-4">
                                <Show when=move || {
                                    matches!(job.result.get(), Some(Ok(())))
                                }>
                                    <div class="bg-green-500/15 border border-green-500/30 \
                                                rounded-xl p-4 flex items-start gap-3">
                                        <span class="text-green-400 text-xl \
                                                     leading-none shrink-0">
                                            "✓"
                                        </span>
                                        <div class="min-w-0">
                                            <div class="text-green-300 font-bold">
                                                "Folder classified"
                                            </div>
                                            <div class="text-green-300/70 text-xs mt-1">
                                                {move || match inspection.get() {
                                                    Some(Ok(i)) if i.has_subdirs => format!(
                                                        "{} collection{} created.",
                                                        i.subdir_names.len(),
                                                        if i.subdir_names.len() == 1 { "" } else { "s" },
                                                    ),
                                                    _ => "Files added to the collection.".to_string(),
                                                }}
                                            </div>
                                        </div>
                                    </div>
                                </Show>

                                <Show when=move || {
                                    matches!(job.result.get(), Some(Err(_)))
                                }>
                                    <ErrorView
                                        error=error_signal
                                        style=ErrorStyle::Banner
                                        heading="Classify failed"
                                    />
                                </Show>

                                <div class="flex justify-end">
                                    <button
                                        type="button"
                                        on:click=move |_| try_close.run(())
                                        class="px-4 py-2 rounded-lg bg-white/10 \
                                               hover:bg-white/20 text-white text-sm \
                                               transition"
                                    >
                                        "Close"
                                    </button>
                                </div>
                            </div>
                        </Show>
                    </div>
                </div>
            }
            .into_any()
        }}
    }
}
