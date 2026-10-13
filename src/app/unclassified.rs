//! Unclassified media browser.
//!
//! Exposes the optional incoming folder (`HOMEX_UNCLASSIFIED_DIR`) as a
//! navigateable tree. Selecting a file opens a picker; confirming moves the
//! file into the chosen collection by reusing the normal upload staging /
//! conversion / persist pipeline.
//!
//! Progress is reported through the shared `state.jobs` map, exactly like the
//! normal upload flow, so the client can show a real progress bar and a
//! final success / failure message.

use gloo_timers::future::TimeoutFuture;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::collections::create_collection_named;
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

// ─── Types ────────────────────────────────────────────────────────────────────

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
        let ext = if is_dir {
            None
        } else {
            name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase())
        };
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
pub async fn classify_unclassified_file(
    rel_path: String,
    collection_id: Uuid,
    title: String,
    season_number: Option<i64>,
) -> Result<String, AppError> {
    use crate::app::model::{CollectionRow, MediaKind, SectionRow};
    use crate::app::server::AppState;
    use crate::app::server::convert::{Job, JobPhase};
    use crate::app::server::upload::{
        UploadFile, UploadPayload, process_upload, schedule_job_eviction,
    };

    let state: AppState = expect_context();
    let root = state
        .config
        .storage
        .unclassified_dir
        .clone()
        .ok_or_else(|| AppError::public("Unclassified media folder is not configured"))?;

    let clean = sanitize_rel(&rel_path)?;
    if clean.is_empty() {
        return Err(AppError::public("No file selected"));
    }
    let source_abs = root.join(&clean);

    let root_canon = tokio::fs::canonicalize(&root)
        .await
        .context("canonicalize unclassified root")?;
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
    let size = meta.len();

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

    let filename = source_canon
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "file".to_string());
    let fallback_title = file_stem(&filename);
    let display_title = {
        let t = title.trim();
        if t.is_empty() {
            fallback_title.clone()
        } else {
            t.to_string()
        }
    };

    let is_video_series = section.nested && kind == MediaKind::Video;
    let season_number = if is_video_series && season_number.is_none() {
        Some(1)
    } else {
        season_number
    };

    leptos::logging::log!(
        "[classify] rel={clean} col={collection_id} season={season_number:?} \
         kind={kind:?} nested={} title={display_title:?}",
        section.nested,
    );

    let temp_dir =
        tempfile::TempDir::new().map_err(|e| AppError::internal("create dummy temp dir", e))?;

    let payload = UploadPayload {
        section_slug: collection.section_slug.clone(),
        collection_id,
        title: collection.title.clone(),
        description: String::new(),
        season_number,
        files: vec![UploadFile {
            filename,
            title: display_title,
            temp_path: source_canon,
            size,
        }],
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

// ─── Client-side job tracker ──────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct ClassifyJob {
    action: Action<(String, Uuid, String, Option<i64>), Result<String, AppError>>,
    pub status: RwSignal<Option<ConversionStatus>>,
    pub result: RwSignal<Option<Result<(), AppError>>>,
    active_job: RwSignal<Option<String>>,
}

impl ClassifyJob {
    pub fn new() -> Self {
        let action = Action::new_local(
            |(path, col, title, season): &(String, Uuid, String, Option<i64>)| {
                classify_unclassified_file(path.clone(), *col, title.clone(), *season)
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

    pub fn dispatch(&self, path: String, col: Uuid, title: String, season: Option<i64>) {
        self.status.set(Some(ConversionStatus::Writing));
        self.result.set(None);
        self.action.dispatch((path, col, title, season));
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
        let rel_path = RwSignal::new(String::new());
        let refresh = RwSignal::new(0u32);
        let selected = RwSignal::new(None::<UnclassifiedEntry>);

        let entries: Resource<Result<Vec<UnclassifiedEntry>, AppError>> = Resource::new(
            move || (rel_path.get(), refresh.get()),
            |(path, _)| list_unclassified(path),
        );

        let classify = ClassifyJob::new();

        let on_modal_close = Callback::new(move |_: ()| {
            refresh.update(|n| *n = n.wrapping_add(1));
        });

        let adapter = move |list: Vec<UnclassifiedEntry>| EntryListProps {
            entries: list,
            rel_path,
            selected,
            classify,
            on_modal_close,
        };

        view! {
            <div class="max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
                <div class="mb-5">
                    <h1 class="text-2xl font-bold text-white">"Unclassified media"</h1>
                    <p class="text-gray-400 text-sm mt-1">
                        "Browse the incoming folder and move files into a collection."
                    </p>
                </div>

                <Breadcrumb rel_path=rel_path/>

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
fn Breadcrumb(rel_path: RwSignal<String>) -> impl IntoView {
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
                                    on:click=move |_| rel_path.set(target.clone())
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
    rel_path: RwSignal<String>,
    selected: RwSignal<Option<UnclassifiedEntry>>,
    classify: ClassifyJob,
    on_modal_close: Callback<()>,
}

fn render_entry_list(props: EntryListProps) -> AnyView {
    let EntryListProps {
        entries,
        rel_path,
        selected,
        classify,
        on_modal_close,
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

    let rows = entries
        .into_iter()
        .map(|entry| {
            if entry.is_dir {
                Either::Left(view! { <DirRow entry=entry rel_path=rel_path/> })
            } else {
                Either::Right(view! { <FileRow entry=entry selected=selected/> })
            }
        })
        .collect_view();

    view! {
        <div>
            <div class="border border-white/10 rounded-xl overflow-hidden bg-white/[0.02]">
                <div class="divide-y divide-white/5">{rows}</div>
            </div>
            <ClassifyModal
                selected=selected
                classify=classify
                on_close=on_modal_close
            />
        </div>
    }
    .into_any()
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
fn DirRow(entry: UnclassifiedEntry, rel_path: RwSignal<String>) -> impl IntoView {
    let target = entry.rel_path.clone();
    let name = entry.name.clone();
    view! {
        <button
            type="button"
            on:click=move |_| rel_path.set(target.clone())
            class="w-full text-left flex items-center gap-3 px-4 py-2.5 hover:bg-white/5 transition"
        >
            <span class="text-cyan-400 shrink-0"><FolderIcon/></span>
            <span class="text-white text-sm truncate flex-1">{name}</span>
            <span class="text-gray-500 text-xs shrink-0">"Open →"</span>
        </button>
    }
}

#[component]
fn FileRow(
    entry: UnclassifiedEntry,
    selected: RwSignal<Option<UnclassifiedEntry>>,
) -> impl IntoView {
    let is_supported = kind_for_ext(entry.ext.as_deref()).is_some();
    let name = entry.name.clone();
    let size = entry.size;
    let entry_for_click = entry;

    let row_class = if is_supported {
        "w-full text-left flex items-center gap-3 px-4 py-2.5 transition hover:bg-white/5"
    } else {
        "w-full text-left flex items-center gap-3 px-4 py-2.5 transition opacity-50 cursor-not-allowed"
    };
    let action_class = if is_supported {
        "text-xs shrink-0 text-cyan-400"
    } else {
        "text-xs shrink-0 text-gray-600"
    };
    let action_label = if is_supported {
        "Classify →"
    } else {
        "Unsupported"
    };

    view! {
        <button
            type="button"
            disabled=!is_supported
            on:click=move |_| selected.set(Some(entry_for_click.clone()))
            class=row_class
        >
            <span class="text-gray-400 shrink-0"><FileIcon/></span>
            <span class="text-white text-sm truncate flex-1">{name}</span>
            <span class="text-gray-500 text-xs font-mono shrink-0">
                {format_size(size)}
            </span>
            <span class=action_class>{action_label}</span>
        </button>
    }
}

// ─── Classify modal ───────────────────────────────────────────────────────────

#[component]
fn ClassifyModal(
    selected: RwSignal<Option<UnclassifiedEntry>>,
    classify: ClassifyJob,
    on_close: Callback<()>,
) -> impl IntoView {
    let file_kind: Memo<Option<MediaKind>> =
        Memo::new(move |_| selected.get().and_then(|e| kind_for_ext(e.ext.as_deref())));

    let sections: Resource<Result<Vec<SectionWithCollections>, AppError>> = Resource::new(
        move || file_kind.get().map(|k| k.as_str().to_string()),
        |k| async move {
            match k {
                Some(s) => fetch_sections_with_collections(s).await,
                None => Ok(Vec::new()),
            }
        },
    );

    let title = RwSignal::new(String::new());
    let chosen_section_slug = RwSignal::new(String::new());
    let chosen_collection_id = RwSignal::new(None::<Uuid>);
    let season = RwSignal::new(String::new());

    // ─── Inline "create new collection" state ─────────────────────────────────

    let creating_new = RwSignal::new(false);
    let new_collection_title = RwSignal::new(String::new());

    let create_action: Action<(String, String), Result<Uuid, AppError>> =
        Action::new_local(|(slug, t): &(String, String)| {
            create_collection_named(slug.clone(), t.clone())
        });

    // On successful create: refetch sections, select the new collection, exit
    // creating mode. The refetched resource causes `chosen_section.get()` to
    // now include the new collection, so the `<select>` re-renders with an
    // option matching `chosen_collection_id`.
    Effect::new(move |_| {
        if let Some(Ok(id)) = create_action.value().get() {
            sections.refetch();
            chosen_collection_id.set(Some(id));
            creating_new.set(false);
            new_collection_title.set(String::new());
        }
    });

    // ─── Reset on file/section change ─────────────────────────────────────────

    Effect::new(move |_| {
        match selected.get() {
            Some(entry) => title.set(file_stem(&entry.name)),
            None => title.set(String::new()),
        }
        chosen_section_slug.set(String::new());
        chosen_collection_id.set(None);
        season.set(String::new());
        creating_new.set(false);
        new_collection_title.set(String::new());
    });

    Effect::new(move |_| {
        if selected.get().is_none() {
            classify.reset();
        }
    });

    // When the section changes, drop the collection selection and close any
    // in-progress "create new" panel.
    Effect::new(move |_| {
        let _ = chosen_section_slug.get();
        chosen_collection_id.set(None);
        creating_new.set(false);
        new_collection_title.set(String::new());
    });

    // ─── Derived signals ──────────────────────────────────────────────────────

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

    // ─── Actions ──────────────────────────────────────────────────────────────

    let try_close = Callback::new(move |_: ()| {
        let busy =
            classify.status.get_untracked().is_some() && classify.result.get_untracked().is_none();
        if busy {
            return;
        }
        let success = matches!(classify.result.get_untracked(), Some(Ok(())));
        selected.set(None);
        if success {
            on_close.run(());
        }
    });

    let confirm = Callback::new(move |_: ()| {
        let Some(entry) = selected.get_untracked() else {
            return;
        };
        let Some(col) = chosen_collection_id.get_untracked() else {
            return;
        };
        let t = title.get_untracked();
        let s = season.get_untracked();
        let s = s.trim();
        let season_num: Option<i64> = if s.is_empty() { None } else { s.parse().ok() };
        classify.dispatch(entry.rel_path.clone(), col, t, season_num);
    });

    // Create-new handlers. Both fire the same action; the success effect above
    // selects the new collection.
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

    // Pre-fill the "new collection" input with the file's stem the first time
    // the user opens the panel — a good guess for a single-file collection.
    Effect::new(move |_| {
        if creating_new.get() && new_collection_title.get_untracked().is_empty() {
            if let Some(entry) = selected.get_untracked() {
                new_collection_title.set(file_stem(&entry.name));
            }
        }
    });

    // ─── Phase signals ────────────────────────────────────────────────────────

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

    // ─── View ─────────────────────────────────────────────────────────────────

    view! {
        {move || {
            let Some(entry) = selected.get() else {
                return ().into_any();
            };
            let entry_name = entry.name.clone();
            let supported = kind_for_ext(entry.ext.as_deref()).is_some();

            view! {
                <div
                    class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm \
                           flex items-center justify-center p-4"
                    on:click=move |_| try_close.run(())
                >
                    <div
                        class="bg-[#12121a] border border-white/10 rounded-2xl p-6 \
                               w-full max-w-lg shadow-2xl"
                        on:click=|ev| ev.stop_propagation()
                    >
                        <div class="flex items-start justify-between gap-3 mb-4">
                            <div class="min-w-0">
                                <h3 class="text-white font-bold text-lg">"Classify file"</h3>
                                <p class="text-gray-400 text-sm truncate mt-0.5">
                                    {entry_name}
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

                        {if !supported {
                            view! {
                                <div class="text-red-300 text-sm py-4 text-center">
                                    "This file type is not supported."
                                </div>
                            }
                            .into_any()
                        } else {
                            view! {
                                // ─── Setup ───
                                <Show when=move || show_setup.get()>
                                    <div class="space-y-4">
                                        <div>
                                            <label class="text-xs text-gray-500 \
                                                          uppercase tracking-wider">
                                                "Title"
                                            </label>
                                            <input
                                                type="text"
                                                prop:value=move || title.get()
                                                on:input=move |e| title.set(event_target_value(&e))
                                                placeholder="Item title"
                                                class="mt-1 w-full bg-white/10 text-white \
                                                       rounded-lg px-3 py-1.5 text-sm \
                                                       focus:outline-none focus:ring-1 \
                                                       focus:ring-cyan-400"
                                            />
                                        </div>

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
                                                <label class="text-xs text-gray-500 \
                                                              uppercase tracking-wider">
                                                    "Collection"
                                                </label>

                                                // Existing collection dropdown
                                                <Show when=move || !creating_new.get()>
                                                    <div class="mt-1 flex items-stretch gap-2">
                                                        <div class="flex-1">
                                                            {move || {
                                                                let Some(s) = chosen_section.get() else {
                                                                    return ().into_any();
                                                                };
                                                                if s.collections.is_empty() {
                                                                    return view! {
                                                                        <div class="text-sm text-gray-500 \
                                                                                    bg-white/5 border border-white/10 \
                                                                                    rounded-lg px-3 py-2">
                                                                            "No collections yet — create one."
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
                                                        </div>
                                                        <button
                                                            type="button"
                                                            on:click=move |_| creating_new.set(true)
                                                            class="shrink-0 px-3 rounded-lg \
                                                                   bg-white/10 hover:bg-white/20 \
                                                                   text-white text-sm transition \
                                                                   whitespace-nowrap"
                                                            title="Create a new collection in this section"
                                                        >
                                                            "+ New"
                                                        </button>
                                                    </div>
                                                </Show>

                                                // Inline "create new collection" form
                                                <Show when=move || creating_new.get()>
                                                    <div class="mt-1 space-y-2">
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
                                                "Move"
                                            </button>
                                        </div>
                                    </div>
                                </Show>

                                // ─── Progress ───
                                <Show when=move || show_progress.get()>
                                    <div class="space-y-3">
                                        <UploadProgress status=status_signal/>
                                        <div class="text-xs text-gray-500 text-center pt-2">
                                            "Please wait — the file is being moved."
                                        </div>
                                    </div>
                                </Show>

                                // ─── Final ───
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
                                                        "Moved successfully"
                                                    </div>
                                                    <div class="text-green-300/70 text-xs mt-1">
                                                        {move || {
                                                            selected
                                                                .get()
                                                                .map(|e| format!(
                                                                    "\"{}\" was added to the collection.",
                                                                    e.name,
                                                                ))
                                                                .unwrap_or_default()
                                                        }}
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
                            }
                            .into_any()
                        }}
                    </div>
                </div>
            }
            .into_any()
        }}
    }
}
