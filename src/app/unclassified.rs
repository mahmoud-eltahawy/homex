//! Unclassified media browser.

mod components;
mod helpers;
mod jobs;
mod tracker;
pub mod types;

use leptos_outcome::ResourceView;
pub use types::{
    ClassifyDestination, ClassifyInput, FileJobHandle, FolderInspection, FoldersInspection,
    SectionWithCollections, UnclassifiedEntry,
};

use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_query_map};
use leptos_router::{LazyRoute, lazy_route};
use uuid::Uuid;

use crate::app::constants::routes;
use crate::app::error::{AppError, ErrorCode};
use crate::app::ids::Slug;
#[cfg(feature = "ssr")]
use crate::app::model::CollectionSummary;
use crate::app::model::MediaKind;
use crate::app::upload_api::ConversionStatus;

use components::breadcrumb::Breadcrumb;
use components::file_modal::ClassifyModal;
use components::folder_modal::FolderClassifyModal;
use components::rows::{DirRow, FileRow};
use components::selection_bar::SelectionBar;
use helpers::encode_query_path;
use jobs::{ClassifyJob, FolderClassifyJob};

// ══════════════════════════════════════════════════════════════════════════
//  Server functions
// ══════════════════════════════════════════════════════════════════════════

#[server]
pub async fn list_unclassified(rel_path: String) -> Result<Vec<UnclassifiedEntry>, AppError> {
    use crate::app::server::AppState;
    use crate::app::server::jobs::ensure_roots_disjoint;
    use crate::app::server::upload::sanitize_rel;

    let state: AppState = expect_context();
    let root = state
        .config
        .storage
        .unclassified_dir
        .clone()
        .ok_or_else(|| AppError::detailed("Unclassified media folder is not configured"))?;

    let root_canon = tokio::fs::canonicalize(&root)
        .await
        .map_err(|e| AppError::internal("canonicalize unclassified root", e))?;
    ensure_roots_disjoint(&root_canon, &state.config.storage.media_root)?;

    let clean = sanitize_rel(&rel_path)?;
    let abs = if clean.is_empty() {
        root.clone()
    } else {
        root.join(&clean)
    };

    let abs_canon = tokio::fs::canonicalize(&abs)
        .await
        .map_err(|e| AppError::detailed(format!("Cannot open folder: {e}")))?;
    if !abs_canon.starts_with(&root_canon) {
        return Err(AppError::public(ErrorCode::InvalidPath));
    }

    let mut rd = tokio::fs::read_dir(&abs_canon)
        .await
        .map_err(|e| AppError::detailed(format!("Cannot read folder: {e}")))?;

    let mut entries = Vec::new();
    while let Some(entry) = rd
        .next_entry()
        .await
        .map_err(|e| AppError::internal("read dir entry", e))?
    {
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
    media_kind: MediaKind,
) -> Result<Vec<SectionWithCollections>, AppError> {
    use crate::app::model::{AudioCollectionRow, SectionRow, VideoCollectionRow};
    use crate::app::server::AppState;

    let state: AppState = expect_context();

    let sections = sqlx::query_as!(
        SectionRow,
        r#"SELECT id            AS "id!: Uuid",
                  slug          AS "slug!: String",
                  title         AS "title!: String",
                  media_kind    AS "media_kind!: String",
                  nested        AS "nested!: bool",
                  position      AS "position!: i64"
           FROM sections WHERE media_kind = ? ORDER BY title ASC"#,
        media_kind.as_str(),
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::internal("fetch sections", e))?;

    let mut out = Vec::with_capacity(sections.len());
    for s in sections {
        let rows: Vec<CollectionSummary> = match media_kind {
            MediaKind::Video => sqlx::query_as!(
                VideoCollectionRow,
                r#"SELECT id AS "id!: Uuid", section_id AS "section_id!: Uuid",
                          section_slug, title, poster, description, position,
                          items_count AS "items_count!: i64"
                   FROM video_collections WHERE section_slug = ? ORDER BY title ASC"#,
                s.slug,
            )
            .fetch_all(&state.db)
            .await
            .map_err(|e| AppError::internal("fetch video collections", e))?
            .into_iter()
            .map(|c| CollectionSummary {
                id: c.id,
                title: c.title,
                section_slug: Slug::from_stored(c.section_slug),
                section_title: s.title.clone(),
                nested: s.nested,
            })
            .collect(),
            MediaKind::Audio => sqlx::query_as!(
                AudioCollectionRow,
                r#"SELECT id AS "id!: Uuid", section_id AS "section_id!: Uuid",
                          section_slug, title, poster, description, position,
                          items_count AS "items_count!: i64"
                   FROM audio_collections WHERE section_slug = ? ORDER BY title ASC"#,
                s.slug,
            )
            .fetch_all(&state.db)
            .await
            .map_err(|e| AppError::internal("fetch audio collections", e))?
            .into_iter()
            .map(|c| CollectionSummary {
                id: c.id,
                title: c.title,
                section_slug: Slug::from_stored(c.section_slug),
                section_title: s.title.clone(),
                nested: s.nested,
            })
            .collect(),
        };

        out.push(SectionWithCollections {
            id: s.id,
            slug: Slug::from_stored(s.slug),
            title: s.title.clone(),
            nested: s.nested,
            collections: rows,
        });
    }
    Ok(out)
}

#[server]
pub async fn inspect_unclassified_folder(rel_path: String) -> Result<FolderInspection, AppError> {
    inspect_folders(vec![rel_path]).await.and_then(|mut v| {
        v.folders
            .pop()
            .ok_or_else(|| AppError::detailed("No folder"))
    })
}

#[server]
pub async fn inspect_unclassified_folders(
    rel_paths: Vec<String>,
) -> Result<FoldersInspection, AppError> {
    inspect_folders(rel_paths).await
}

#[cfg(feature = "ssr")]
async fn inspect_folders(rel_paths: Vec<String>) -> Result<FoldersInspection, AppError> {
    use crate::app::server::AppState;
    use crate::app::server::jobs::ensure_roots_disjoint;
    use crate::app::server::upload::{
        collect_files, dominant_kind, read_immediate_subdirs, sanitize_rel,
    };

    if rel_paths.is_empty() {
        return Err(AppError::public(ErrorCode::NoFoldersSelected));
    }
    let state: AppState = expect_context();
    let root = state
        .config
        .storage
        .unclassified_dir
        .clone()
        .ok_or_else(|| AppError::detailed("Unclassified media folder is not configured"))?;
    let root_canon = tokio::fs::canonicalize(&root)
        .await
        .map_err(|e| AppError::internal("canonicalize root", e))?;
    ensure_roots_disjoint(&root_canon, &state.config.storage.media_root)?;

    let mut folders = Vec::with_capacity(rel_paths.len());
    for rp in &rel_paths {
        let clean = sanitize_rel(rp)?;
        if clean.is_empty() {
            return Err(AppError::public(ErrorCode::InvalidPath));
        }
        let abs_canon = tokio::fs::canonicalize(root.join(&clean))
            .await
            .map_err(|e| AppError::detailed(format!("Folder not found: {e}")))?;
        if !abs_canon.starts_with(&root_canon) {
            return Err(AppError::public(ErrorCode::InvalidPath));
        }
        if !tokio::fs::metadata(&abs_canon)
            .await
            .map_err(|e| AppError::internal("stat folder", e))?
            .is_dir()
        {
            return Err(AppError::public(ErrorCode::NotAFolder));
        }
        let name = abs_canon
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Folder".to_string());
        let subdir_names = read_immediate_subdirs(&abs_canon).await?;
        let has_subdirs = !subdir_names.is_empty();
        let files = collect_files(&root_canon, &abs_canon, has_subdirs).await?;
        folders.push(FolderInspection {
            rel_path: clean,
            name,
            has_subdirs,
            subdir_names,
            file_count: files.len(),
            total_size: files.iter().map(|f| f.size).sum(),
            dominant_kind: dominant_kind(&files),
        });
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

// ─── Classify: files ──────────────────────────────────────────────────────

#[server]
pub async fn classify_unclassified_files(
    files: Vec<ClassifyInput>,
    destination: ClassifyDestination,
    season_number: Option<i64>,
) -> Result<Vec<FileJobHandle>, AppError> {
    use crate::app::model::{MediaKind, SectionRow};
    use crate::app::server::jobs::{
        acquire_classify_locks, ensure_roots_disjoint, prepare_file_job, register_and_spawn,
        spawn_lock_holder,
    };
    use crate::app::server::upload::{UploadTarget, extract_poster_for_new_video_collection};
    use crate::app::server::{AppState, db};

    if files.is_empty() {
        return Err(AppError::public(ErrorCode::NoFilesReceived));
    }

    let state: AppState = expect_context();
    let root = state
        .config
        .storage
        .unclassified_dir
        .clone()
        .ok_or_else(|| AppError::detailed("Unclassified media folder is not configured"))?;
    let root_canon = tokio::fs::canonicalize(&root)
        .await
        .map_err(|e| AppError::internal("canonicalize root", e))?;
    ensure_roots_disjoint(&root_canon, &state.config.storage.media_root)?;

    let (target, section_slug, section_kind, section_nested) = match &destination {
        ClassifyDestination::Existing(id) => {
            let collection = db::fetch_collection_any(&state.db, *id)
                .await
                .map_err(|e| AppError::internal("load collection", e))?
                .ok_or_else(|| AppError::public(ErrorCode::CollectionNotFound))?;
            let section = sqlx::query_as!(
                SectionRow,
                r#"SELECT id            AS "id!: Uuid",
                          slug          AS "slug!: String",
                          title         AS "title!: String",
                          media_kind    AS "media_kind!: String",
                          nested        AS "nested!: bool",
                          position      AS "position!: i64"
                   FROM sections WHERE slug = ?"#,
                collection.section_slug.as_str(),
            )
            .fetch_one(&state.db)
            .await
            .map_err(|e| AppError::internal("load section", e))?;
            let kind = MediaKind::try_from(section.media_kind.as_str())
                .map_err(|e| AppError::internal("section has invalid media_kind", e))?;
            (
                UploadTarget::Existing { collection_id: *id },
                collection.section_slug.clone(),
                kind,
                section.nested,
            )
        }
        ClassifyDestination::New {
            section_slug,
            title,
        } => {
            let section = sqlx::query_as!(
                SectionRow,
                r#"SELECT id            AS "id!: Uuid",
                          slug          AS "slug!: String",
                          title         AS "title!: String",
                          media_kind    AS "media_kind!: String",
                          nested        AS "nested!: bool",
                          position      AS "position!: i64"
                   FROM sections WHERE slug = ?"#,
                section_slug.as_str(),
            )
            .fetch_one(&state.db)
            .await
            .map_err(|e| AppError::internal("load section", e))?;
            let kind = MediaKind::try_from(section.media_kind.as_str())
                .map_err(|e| AppError::internal("section has invalid media_kind", e))?;
            let id = Uuid::new_v4();
            (
                UploadTarget::New {
                    id,
                    title: title.clone(),
                    description: None,
                    tags: Vec::new(),
                },
                section_slug.clone(),
                kind,
                section.nested,
            )
        }
    };

    let mut upload_files = Vec::with_capacity(files.len());
    for input in &files {
        upload_files.push(resolve_upload_file(&root, &root_canon, input).await?);
    }

    let paths: Vec<std::path::PathBuf> = upload_files.iter().map(|f| f.temp_path.clone()).collect();
    let lock = acquire_classify_locks(paths)?;

    let season_number =
        if section_nested && section_kind == MediaKind::Video && season_number.is_none() {
            Some(1)
        } else {
            season_number
        };

    if let UploadTarget::New { id, title, .. } = &target {
        let id = *id;
        match section_kind {
            MediaKind::Video => {
                let first = upload_files
                    .first()
                    .ok_or_else(|| AppError::public(ErrorCode::NoFilesReceived))?;
                let url = extract_poster_for_new_video_collection(
                    &state,
                    &section_slug,
                    id,
                    &first.temp_path,
                    true,
                )
                .await?;
                db::insert_video_collection(&state.db, id, &section_slug, title, None, &[], &url)
                    .await
                    .map_err(|e| AppError::internal("insert video collection", e))?;
            }
            MediaKind::Audio => {
                db::insert_audio_collection(&state.db, id, &section_slug, title, None, &[], None)
                    .await
                    .map_err(|e| AppError::internal("insert audio collection", e))?;
            }
        }
    }

    let collection_id = match &target {
        UploadTarget::Existing { collection_id } => *collection_id,
        UploadTarget::New { id, .. } => *id,
    };
    let target_for_jobs = UploadTarget::Existing { collection_id };

    let mut prepared = Vec::with_capacity(upload_files.len());
    for (input, file) in files.into_iter().zip(upload_files) {
        prepared.push(prepare_file_job(
            file,
            target_for_jobs.clone(),
            &section_slug,
            season_number,
            section_kind,
            input.rel_path,
        )?);
    }

    let handles = register_and_spawn(&state, prepared, &section_slug, Some(collection_id)).await;
    let job_ids: Vec<String> = handles.iter().map(|h| h.job_id.clone()).collect();
    spawn_lock_holder(state.clone(), job_ids, lock);

    Ok(handles)
}

// ─── Classify: folders ────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
const MAX_CLASSIFY_GROUPS: usize = 50;

#[server]
pub async fn classify_unclassified_folder(
    rel_path: String,
    destination: ClassifyDestination,
) -> Result<Vec<FileJobHandle>, AppError> {
    classify_unclassified_folders(vec![rel_path], destination).await
}

#[server]
pub async fn classify_unclassified_folders(
    rel_paths: Vec<String>,
    destination: ClassifyDestination,
) -> Result<Vec<FileJobHandle>, AppError> {
    use crate::app::model::{MediaKind, SectionRow};
    use crate::app::server::jobs::{
        acquire_classify_locks, ensure_roots_disjoint, prepare_file_job, register_and_spawn,
        spawn_cleanup_watcher, spawn_lock_holder,
    };
    use crate::app::server::upload::{
        RawFile, UploadTarget, collect_files, ensure_kind_matches,
        extract_poster_for_new_video_collection, read_immediate_subdirs, sanitize_rel,
    };
    use crate::app::server::{AppState, db};

    if rel_paths.is_empty() {
        return Err(AppError::public(ErrorCode::NoFoldersSelected));
    }

    let state: AppState = expect_context();
    let root = state
        .config
        .storage
        .unclassified_dir
        .clone()
        .ok_or_else(|| AppError::detailed("Unclassified media folder is not configured"))?;
    let root_canon = tokio::fs::canonicalize(&root)
        .await
        .map_err(|e| AppError::internal("canonicalize root", e))?;
    ensure_roots_disjoint(&root_canon, &state.config.storage.media_root)?;

    let mut abs_canons: Vec<std::path::PathBuf> = Vec::with_capacity(rel_paths.len());
    for rp in &rel_paths {
        let clean = sanitize_rel(rp)?;
        if clean.is_empty() {
            return Err(AppError::public(ErrorCode::InvalidPath));
        }
        let abs_canon = tokio::fs::canonicalize(root.join(&clean))
            .await
            .map_err(|e| AppError::detailed(format!("Folder not found: {e}")))?;
        if !abs_canon.starts_with(&root_canon) {
            return Err(AppError::public(ErrorCode::InvalidPath));
        }
        if !tokio::fs::metadata(&abs_canon)
            .await
            .map_err(|e| AppError::internal("stat folder", e))?
            .is_dir()
        {
            return Err(AppError::public(ErrorCode::NotAFolder));
        }
        abs_canons.push(abs_canon);
    }
    let lock = acquire_classify_locks(abs_canons.clone())?;

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

    enum Dest {
        Existing {
            slug: Slug,
            kind: MediaKind,
            nested: bool,
        },
        New {
            title: String,
            kind: MediaKind,
            nested: bool,
        },
    }

    let dest = match destination.clone() {
        ClassifyDestination::Existing(id) => {
            let collection = db::fetch_collection_any(&state.db, id)
                .await
                .map_err(|e| AppError::internal("load collection", e))?
                .ok_or_else(|| AppError::public(ErrorCode::CollectionNotFound))?;
            let section = sqlx::query_as!(
                SectionRow,
                r#"SELECT id            AS "id!: Uuid",
                          slug          AS "slug!: String",
                          title         AS "title!: String",
                          media_kind    AS "media_kind!: String",
                          nested        AS "nested!: bool",
                          position      AS "position!: i64"
                   FROM sections WHERE slug = ?"#,
                collection.section_slug.as_str(),
            )
            .fetch_one(&state.db)
            .await
            .map_err(|e| AppError::internal("load section", e))?;
            let kind = MediaKind::try_from(section.media_kind.as_str())
                .map_err(|e| AppError::internal("section has invalid media_kind", e))?;
            if any_has_subdirs && !section.nested {
                return Err(AppError::detailed(
                    "At least one folder contains subfolders — choose a nested section.",
                ));
            }
            Dest::Existing {
                slug: collection.section_slug.clone(),
                kind,
                nested: section.nested,
            }
        }
        ClassifyDestination::New {
            section_slug,
            title,
        } => {
            let title = title.trim().to_string();
            if title.is_empty() {
                return Err(AppError::validation(ErrorCode::TitleRequired));
            }
            let section = sqlx::query_as!(
                SectionRow,
                r#"SELECT id            AS "id!: Uuid",
                          slug          AS "slug!: String",
                          title         AS "title!: String",
                          media_kind    AS "media_kind!: String",
                          nested        AS "nested!: bool",
                          position      AS "position!: i64"
                   FROM sections WHERE slug = ?"#,
                section_slug.as_str(),
            )
            .fetch_one(&state.db)
            .await
            .map_err(|e| AppError::internal("load section", e))?;
            let kind = MediaKind::try_from(section.media_kind.as_str())
                .map_err(|e| AppError::internal("section has invalid media_kind", e))?;
            Dest::New {
                title,
                kind,
                nested: section.nested,
            }
        }
    };

    let (kind, nested) = match &dest {
        Dest::Existing { kind, nested, .. } => (*kind, *nested),
        Dest::New { kind, nested, .. } => (*kind, *nested),
    };

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
        return Err(AppError::public(ErrorCode::NoFilesFound));
    }
    if groups.len() > MAX_CLASSIFY_GROUPS {
        return Err(AppError::public(ErrorCode::TooManyFolders {
            count: groups.len(),
            max: MAX_CLASSIFY_GROUPS,
        }));
    }

    let existing = match &dest {
        Dest::Existing { slug, .. } => existing_collection_titles(&state.db, slug, kind).await?,
        Dest::New { .. } => Default::default(),
    };
    let before = groups.len();
    let mut kept = Vec::with_capacity(before);
    for g in groups {
        if existing.contains_key(&g.title) {
            continue;
        }
        kept.push(g);
    }
    let skipped = before - kept.len();
    if kept.is_empty() {
        return Err(AppError::detailed(format!(
            "All {before} collection{} already exist in the destination section.",
            if before == 1 { "" } else { "s" },
        )));
    }
    if skipped > 0 {
        leptos::logging::log!(
            "[classify] skipped {skipped} collection{} already classified",
            if skipped == 1 { "" } else { "s" },
        );
    }
    let groups = kept;

    let section_slug: Slug = match &dest {
        Dest::Existing { slug, .. } => slug.clone(),
        Dest::New { title, .. } => {
            let slug = generate_unique_slug(&state.db, kind, nested).await?;
            db::insert_section(&state.db, &slug, title, kind, nested)
                .await
                .map_err(|e| AppError::internal("insert section", e))?;
            slug
        }
    };

    let mut prepared = Vec::new();
    let mut cleanup_watchers: Vec<(Vec<String>, std::path::PathBuf)> = Vec::new();

    for group in groups {
        let collection_id = Uuid::new_v4();

        match kind {
            MediaKind::Video => {
                let first = group
                    .files
                    .first()
                    .ok_or_else(|| AppError::detailed("Empty group"))?;
                let url = extract_poster_for_new_video_collection(
                    &state,
                    &section_slug,
                    collection_id,
                    &first.abs,
                    true,
                )
                .await?;
                db::insert_video_collection(
                    &state.db,
                    collection_id,
                    &section_slug,
                    &group.title,
                    None,
                    &[],
                    &url,
                )
                .await
                .map_err(|e| AppError::internal("insert video collection", e))?;
            }
            MediaKind::Audio => {
                db::insert_audio_collection(
                    &state.db,
                    collection_id,
                    &section_slug,
                    &group.title,
                    None,
                    &[],
                    None,
                )
                .await
                .map_err(|e| AppError::internal("insert audio collection", e))?;
            }
        }

        let mut group_job_ids = Vec::new();
        for raw in group.files {
            let upload = crate::app::server::upload::UploadFile {
                title: helpers::file_stem(&raw.filename),
                filename: raw.filename,
                temp_path: raw.abs,
                size: raw.size,
            };
            let p = prepare_file_job(
                upload,
                UploadTarget::Existing { collection_id },
                &section_slug,
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
    spawn_lock_holder(state.clone(), all_job_ids, lock);

    for (job_ids, dir) in cleanup_watchers {
        spawn_cleanup_watcher(state.clone(), job_ids, dir);
    }

    Ok(handles)
}

// ─── Move items back to incoming ──────────────────────────────────────────

#[server]
pub async fn move_items_to_incoming(item_ids: Vec<Uuid>) -> Result<(), AppError> {
    use crate::app::model::{File, Item};
    use crate::app::server::jobs::canonical_media_root;
    use crate::app::server::upload::move_file_to_incoming;
    use crate::app::server::{AppState, db, remove_poster};

    if item_ids.is_empty() {
        return Ok(());
    }

    let state: AppState = expect_context();
    let incoming = state
        .config
        .storage
        .unclassified_dir
        .clone()
        .ok_or_else(|| AppError::detailed("Unclassified media folder is not configured"))?;
    tokio::fs::create_dir_all(&incoming)
        .await
        .map_err(|e| AppError::internal("mkdir incoming", e))?;
    let incoming_canon = tokio::fs::canonicalize(&incoming)
        .await
        .map_err(|e| AppError::internal("canonicalize incoming", e))?;
    let media_canon = canonical_media_root(&state.config.storage.media_root)?;

    let mut moved: Vec<Uuid> = Vec::new();

    for item_id in item_ids {
        let item = match sqlx::query_as!(
            Item,
            r#"SELECT id AS "id!: Uuid", collection_id AS "collection_id!: Uuid",
                      number AS "number!: i64", season_number, title, poster,
                      description, file_id AS "file_id!: Uuid"
               FROM items WHERE id = ?"#,
            item_id,
        )
        .fetch_optional(&state.db)
        .await
        {
            Ok(Some(i)) => i,
            _ => continue,
        };
        let file = match sqlx::query_as!(
            File,
            r#"SELECT id AS "id!: Uuid", relative_path, size_bytes, duration_secs
               FROM files WHERE id = ?"#,
            item.file_id,
        )
        .fetch_optional(&state.db)
        .await
        {
            Ok(Some(f)) => f,
            _ => continue,
        };
        match move_file_to_incoming(media_canon, &incoming_canon, &file.relative_path).await {
            Ok(Some(_)) => moved.push(item_id),
            Ok(None) => {}
            Err(e) => leptos::logging::warn!("[move_to_incoming] {e}"),
        }
    }

    if !moved.is_empty() {
        let del = db::delete_items_cascade(&state.db, &moved)
            .await
            .map_err(|e| AppError::internal("delete items cascade", e))?;
        for poster in &del.emptied_posters {
            remove_poster(&state, Some(poster.as_str())).await;
        }
    }
    Ok(())
}

// ══════════════════════════════════════════════════════════════════════════
//  SSR-only helpers
// ══════════════════════════════════════════════════════════════════════════

#[cfg(feature = "ssr")]
async fn resolve_upload_file(
    root: &std::path::Path,
    root_canon: &std::path::Path,
    input: &ClassifyInput,
) -> Result<crate::app::server::upload::UploadFile, AppError> {
    use crate::app::server::upload::{UploadFile, sanitize_rel};

    let clean = sanitize_rel(&input.rel_path)?;
    if clean.is_empty() {
        return Err(AppError::detailed("Empty file path"));
    }
    let source_canon = tokio::fs::canonicalize(root.join(&clean))
        .await
        .map_err(|e| AppError::detailed(format!("File not found: {e}")))?;
    if !source_canon.starts_with(root_canon) {
        return Err(AppError::public(ErrorCode::InvalidPath));
    }
    let meta = tokio::fs::metadata(&source_canon)
        .await
        .map_err(|e| AppError::internal("stat source file", e))?;
    if !meta.is_file() {
        return Err(AppError::public(ErrorCode::NotAFile));
    }
    let filename = source_canon
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let title = {
        let t = input.title.trim();
        if t.is_empty() {
            helpers::file_stem(&filename)
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
async fn existing_collection_titles(
    db: &sqlx::SqlitePool,
    section_slug: &Slug,
    kind: crate::app::model::MediaKind,
) -> Result<std::collections::HashMap<String, Uuid>, AppError> {
    use std::collections::HashMap;

    let rows: Vec<(String, Uuid)> = match kind {
        crate::app::model::MediaKind::Video => sqlx::query_as!(
            _TitleId,
            r#"SELECT title AS "title!: String", id AS "id!: Uuid"
               FROM video_collections WHERE section_slug = ?"#,
            section_slug.as_str(),
        )
        .fetch_all(db)
        .await
        .map_err(|e| AppError::internal("load existing collections", e))?
        .into_iter()
        .map(|r| (r.title, r.id))
        .collect(),
        crate::app::model::MediaKind::Audio => sqlx::query_as!(
            _TitleId,
            r#"SELECT title AS "title!: String", id AS "id!: Uuid"
               FROM audio_collections WHERE section_slug = ?"#,
            section_slug.as_str(),
        )
        .fetch_all(db)
        .await
        .map_err(|e| AppError::internal("load existing collections", e))?
        .into_iter()
        .map(|r| (r.title, r.id))
        .collect(),
    };
    Ok(rows.into_iter().collect::<HashMap<_, _>>())
}

#[cfg(feature = "ssr")]
#[derive(sqlx::FromRow)]
struct _TitleId {
    title: String,
    id: Uuid,
}

#[cfg(feature = "ssr")]
async fn generate_unique_slug(
    db: &sqlx::SqlitePool,
    kind: crate::app::model::MediaKind,
    nested: bool,
) -> Result<Slug, AppError> {
    use crate::app::server::db;
    let prefix = crate::app::sections::prefix_for(kind, nested);
    for _ in 0..4 {
        let candidate = format!("{prefix}-{}", crate::app::sections::short_hash());
        let slug = Slug::new(candidate).map_err(|e| {
            AppError::internal("create section", format!("generated invalid slug: {e}"))
        })?;
        if db::fetch_section_by_slug(db, &slug)
            .await
            .map_err(|e| AppError::internal("check slug uniqueness", e))?
            .is_none()
        {
            return Ok(slug);
        }
    }
    Err(AppError::internal(
        "create section",
        "could not generate a unique section identifier",
    ))
}

// ══════════════════════════════════════════════════════════════════════════
//  Entry list bridging
// ══════════════════════════════════════════════════════════════════════════

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
                Either::Left(view! { <DirRow entry on_navigate folder_selection/> })
            } else {
                Either::Right(view! { <FileRow entry file_selection modal_open/> })
            }
        })
        .collect_view();

    view! {
        <div>
            <SelectionBar
                entries=entries_for_bar
                file_selection
                folder_selection
                modal_open
                folder_modal_open
            />
            <div class="border border-white/10 rounded-xl overflow-hidden bg-white/[0.02]">
                <div class="divide-y divide-white/5">{rows}</div>
            </div>
            <ClassifyModal
                selection=file_selection
                open=modal_open
                job=classify
                on_close=on_modal_close
            />
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

// ══════════════════════════════════════════════════════════════════════════
//  Page
// ══════════════════════════════════════════════════════════════════════════

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
