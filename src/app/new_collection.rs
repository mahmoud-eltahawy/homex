//! Draft page for creating a new collection.
//!
//! Nothing is persisted on the server until the upload is submitted. The
//! collection row is created inside `upload_media` after staging — for
//! video, only after a poster frame is extracted.

use crate::app::constants::routes;
use crate::app::error::AppError;
use crate::app::model::{MediaKind, Section, Tag};
use crate::app::route_params::use_string_param;
use crate::app::tags::TagEditor;
use crate::app::upload_job::{UploadJob, UploadProgress};
use leptos::prelude::*;
use leptos_outcome::ErrorView;
use leptos_router::hooks::use_navigate;
use leptos_router::{LazyRoute, lazy_route};

// ─── FormData field names ─────────────────────────────────────────────────

mod fields {
    pub const SECTION_SLUG: &str = "section_slug";
    pub const COLLECTION_TITLE: &str = "collection_title";
    pub const DESCRIPTION: &str = "description";
    pub const TAG_PREFIX: &str = "tag_";
    pub const SEASON_NUMBER: &str = "season_number";
    pub const POSTER_FILE: &str = "poster_file";
    pub const FILE_PREFIX: &str = "file_";
    pub const FILE_TITLE_PREFIX: &str = "file_title_";

    pub fn file(idx: usize) -> String {
        format!("{FILE_PREFIX}{idx}")
    }
    pub fn file_title(idx: usize) -> String {
        format!("{FILE_TITLE_PREFIX}{idx}")
    }
    pub fn tag(idx: usize) -> String {
        format!("{TAG_PREFIX}{idx}")
    }
}

// ─── Page ─────────────────────────────────────────────────────────────────

pub struct NewCollectionPage {
    section_slug: Signal<String>,
    sections: Resource<Result<Vec<Section>, AppError>>,
}

#[lazy_route]
impl LazyRoute for NewCollectionPage {
    fn data() -> Self {
        let slug_fn = use_string_param("slug");
        let section_slug: Signal<String> = Signal::derive(slug_fn);
        let sections: Resource<Result<Vec<Section>, AppError>> = expect_context();
        Self {
            section_slug,
            sections,
        }
    }

    fn view(this: Self) -> AnyView {
        let section_slug = this.section_slug;
        let sections = this.sections;
        let navigate = use_navigate();

        let section: Signal<Option<Section>> = Signal::derive(move || {
            let slug = section_slug.get();
            sections
                .get()
                .and_then(|r| r.ok())
                .and_then(|v| v.into_iter().find(|s| s.slug == slug))
        });

        let kind = Signal::derive(move || section.get().map(|s| s.media_kind()));
        let is_audio = Signal::derive(move || kind.get() == Some(MediaKind::Audio));
        let is_series = Signal::derive(move || {
            matches!(
                section.get(),
                Some(s) if s.nested && s.media_kind() == MediaKind::Video
            )
        });

        let title = RwSignal::new(String::new());
        let description = RwSignal::new(String::new());
        let tags: RwSignal<Vec<Tag>> = RwSignal::new(Vec::new());
        let season = RwSignal::new("1".to_string());
        let poster_file = RwSignal::new(None::<web_sys::File>);
        let files = RwSignal::new(Vec::<web_sys::File>::new());

        let upload = UploadJob::new_for_new_collection();
        let upload_pending = upload.pending;
        let upload_status = upload.status;
        let upload_error = upload.error();

        let navigate_for_effect = navigate.clone();
        let slug_for_effect = section_slug;
        Effect::new(move |_| {
            if !upload.is_terminal_success() {
                return;
            }
            let Some(id) = upload.collection_id.get() else {
                return;
            };
            let Some(slug) = upload.section_slug.get().or_else(|| {
                let s = slug_for_effect.get();
                if s.is_empty() { None } else { Some(s) }
            }) else {
                return;
            };
            navigate_for_effect(&routes::collection(&slug, id), Default::default());
        });

        let can_submit = Signal::derive(move || {
            !title.get().trim().is_empty() && !files.get().is_empty() && !upload_pending.get()
        });

        let accept = Signal::derive(move || match kind.get() {
            Some(MediaKind::Audio) => ".mp3,.m4a,.flac,.wav,.ogg,.opus,.aac",
            _ => ".mp4,.mkv,.mov,.webm,.avi,.m4v,.wmv,.flv,.ts",
        });

        let on_submit = move |ev: web_sys::SubmitEvent| {
            ev.prevent_default();
            let slug = section_slug.get_untracked();
            if slug.is_empty() {
                return;
            }
            let t = title.get_untracked().trim().to_string();
            if t.is_empty() {
                return;
            }
            let desc = description.get_untracked();
            let tag_names: Vec<String> = tags.get_untracked().into_iter().map(|t| t.name).collect();
            let season_opt = if is_series.get_untracked() {
                season.get_untracked().trim().parse::<i64>().ok()
            } else {
                None
            };
            let poster_opt = if is_audio.get_untracked() {
                poster_file.get_untracked()
            } else {
                None
            };

            let fd = build_formdata(
                &slug,
                &t,
                if desc.trim().is_empty() {
                    None
                } else {
                    Some(&desc)
                },
                &tag_names,
                season_opt,
                poster_opt.as_ref(),
                &files.get_untracked(),
            );
            upload.dispatch(fd);
        };

        view! {
            <div class="max-w-2xl mx-auto px-4 sm:px-6 lg:px-8 py-6">
                {move || match section.get() {
                    None => view! {
                        <div class="py-12 text-center text-gray-500 text-sm">
                            "Loading section…"
                        </div>
                    }.into_any(),
                    Some(s) => {
                        let label = s.new_label().to_string();
                        view! {
                            <div>
                                <h1 class="text-2xl font-bold text-white mb-1">{label}</h1>
                                <p class="text-gray-400 text-sm mb-6">
                                    "The collection is created once the upload succeeds."
                                </p>

                                <form on:submit=on_submit class="space-y-4">
                                    <div>
                                        <label class="text-xs text-gray-500 uppercase tracking-wider">
                                            "Title"
                                        </label>
                                        <input
                                            type="text"
                                            prop:value=move || title.get()
                                            on:input=move |e| title.set(event_target_value(&e))
                                            autofocus=true
                                            placeholder="Collection title"
                                            class="mt-1 w-full bg-white/10 text-white rounded-lg \
                                                   px-3 py-2 text-sm focus:outline-none \
                                                   focus:ring-1 focus:ring-cyan-400"
                                        />
                                    </div>

                                    <div>
                                        <label class="text-xs text-gray-500 uppercase tracking-wider">
                                            "Description"
                                        </label>
                                        <textarea
                                            rows=3
                                            prop:value=move || description.get()
                                            on:input=move |e| description.set(event_target_value(&e))
                                            placeholder="Optional"
                                            class="mt-1 w-full bg-white/10 text-white rounded-lg \
                                                   px-3 py-2 text-sm focus:outline-none \
                                                   focus:ring-1 focus:ring-cyan-400 resize-none"
                                        ></textarea>
                                    </div>

                                    <div>
                                        <label class="text-xs text-gray-500 uppercase tracking-wider">
                                            "Tags"
                                        </label>
                                        <div class="mt-1">
                                            <TagEditor
                                                tags=tags
                                                on_commit=Callback::new(|_: Vec<String>| ())
                                            />
                                        </div>
                                    </div>

                                    <Show when=move || is_series.get()>
                                        <div>
                                            <label class="text-xs text-gray-500 uppercase tracking-wider">
                                                "Season"
                                            </label>
                                            <input
                                                type="number"
                                                min="1"
                                                prop:value=move || season.get()
                                                on:input=move |e| season.set(event_target_value(&e))
                                                class="mt-1 w-full bg-white/10 text-white rounded-lg \
                                                       px-3 py-2 text-sm focus:outline-none \
                                                       focus:ring-1 focus:ring-cyan-400"
                                            />
                                        </div>
                                    </Show>

                                    <Show when=move || is_audio.get()>
                                        <div>
                                            <label class="text-xs text-gray-500 uppercase tracking-wider">
                                                "Poster (optional)"
                                            </label>
                                            <input
                                                type="file"
                                                accept="image/jpeg,image/png,image/webp"
                                                on:change=move |e| {
                                                    let fs = event_target_files(&e);
                                                    poster_file.set(fs.into_iter().next());
                                                }
                                                class="mt-1 w-full text-white text-sm"
                                            />
                                        </div>
                                    </Show>

                                    <div>
                                        <label class="text-xs text-gray-500 uppercase tracking-wider">
                                            "Files"
                                        </label>
                                        <input
                                            type="file"
                                            multiple=true
                                            accept=move || accept.get()
                                            on:change=move |e| {
                                                files.set(event_target_files(&e));
                                            }
                                            class="mt-1 w-full text-white text-sm"
                                        />
                                        <p class="text-xs text-gray-500 mt-1">
                                            {move || format!(
                                                "{} file(s) selected",
                                                files.get().len()
                                            )}
                                        </p>
                                    </div>

                                    <div class="flex items-center gap-3 pt-2">
                                        <button
                                            type="submit"
                                            disabled=move || !can_submit.get()
                                            class="px-4 py-2 rounded-lg bg-gradient-to-r \
                                                   from-cyan-500 to-blue-500 \
                                                   hover:from-cyan-400 hover:to-blue-400 \
                                                   text-white font-bold text-sm \
                                                   disabled:opacity-50 transition"
                                        >
                                            {move || if upload_pending.get() {
                                                "Uploading…".to_string()
                                            } else {
                                                "Upload".to_string()
                                            }}
                                        </button>
                                    </div>
                                </form>

                                <div class="mt-4">
                                    <UploadProgress status=Signal::derive(move || upload_status.get())/>
                                </div>
                                <div class="mt-3">
                                    <ErrorView error=upload_error banner=true/>
                                </div>
                            </div>
                        }.into_any()
                    }
                }}
            </div>
        }
        .into_any()
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn event_target_files(ev: &web_sys::Event) -> Vec<web_sys::File> {
    use web_sys::wasm_bindgen::JsCast;
    let Some(input) = ev
        .target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
    else {
        return Vec::new();
    };
    let Some(list) = input.files() else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(list.length() as usize);
    for i in 0..list.length() {
        if let Some(f) = list.get(i) {
            out.push(f.unchecked_into::<web_sys::File>());
        }
    }
    input.set_value("");
    out
}

#[allow(clippy::too_many_arguments)]
fn build_formdata(
    section_slug: &str,
    title: &str,
    description: Option<&str>,
    tags: &[String],
    season: Option<i64>,
    poster: Option<&web_sys::File>,
    files: &[web_sys::File],
) -> web_sys::FormData {
    let fd = web_sys::FormData::new().unwrap();
    let _ = fd.append_with_str(fields::SECTION_SLUG, section_slug);
    let _ = fd.append_with_str(fields::COLLECTION_TITLE, title);
    if let Some(d) = description {
        let _ = fd.append_with_str(fields::DESCRIPTION, d);
    }
    for (i, t) in tags.iter().enumerate() {
        let _ = fd.append_with_str(&fields::tag(i), t);
    }
    if let Some(s) = season {
        let _ = fd.append_with_str(fields::SEASON_NUMBER, &s.to_string());
    }
    if let Some(p) = poster {
        let _ = fd.append_with_blob_and_filename(fields::POSTER_FILE, p, &p.name());
    }
    for (i, f) in files.iter().enumerate() {
        let name = f.name();
        let stem = name.rsplitn(2, '.').last().unwrap_or(&name).to_string();
        let _ = fd.append_with_blob_and_filename(&fields::file(i), f, &name);
        let _ = fd.append_with_str(&fields::file_title(i), &stem);
    }
    fd
}
