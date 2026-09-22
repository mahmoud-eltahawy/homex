use std::collections::BTreeSet;

use leptos::{either::Either, prelude::*};
use leptos_router::{LazyRoute, lazy_route};
use web_sys::wasm_bindgen::JsCast;

use crate::app::{
    collections::{
        delete_item, fetch_collection_detail, fetch_items, patch_collection_field,
        patch_item_title, upload_collection_poster,
    },
    common::refetch_on_success,
    detail::{DetailHero, DetailShell, HeroBadge, HeroMeta},
    icons::{ClockIcon, MoviePosterSvg, MusicPosterSvg, UploadIcon, icon_for},
    inline_edit::{EditablePoster, EditableText, EditableTextArea, use_edit_mode},
    media_player::{MediaItem, MediaPlayer},
    model::{Collection, Item, MediaKind, Section},
    resource_view::ResourceView,
    route_params::{use_optional_u64_param, use_string_param, use_u64_param},
    sections::fetch_section_by_slug,
    upload_job::{UploadJob, UploadProgress},
};

// ─── Action type aliases ──────────────────────────────────────────────────

type PatchAction = Action<(u64, String, Option<String>), Result<(), ServerFnError>>;
type PosterUploadAction = Action<web_sys::FormData, Result<String, ServerFnError>>;
type RenameItemAction = Action<(u64, String), Result<(), ServerFnError>>;
type DeleteItemAction = Action<u64, Result<(), ServerFnError>>;

// ─── Collection actions bundle ────────────────────────────────────────────

#[derive(Clone, Copy)]
struct CollectionActions {
    patch: PatchAction,
    poster_upload: PosterUploadAction,
    rename_item: RenameItemAction,
    delete_item: DeleteItemAction,
}

impl CollectionActions {
    /// Builds the four collection/item actions and wires the two that
    /// affect the items list into a refetch.
    fn new(items: Resource<Result<Vec<Item>, ServerFnError>>) -> Self {
        let patch = Action::new_local(|(id, field, value): &(u64, String, Option<String>)| {
            patch_collection_field(*id, field.clone(), value.clone())
        });
        let poster_upload =
            Action::new_local(|fd: &web_sys::FormData| upload_collection_poster(fd.clone().into()));
        let rename_item =
            Action::new_local(|(id, t): &(u64, String)| patch_item_title(*id, t.clone()));
        let delete_item = Action::new_local(|id: &u64| delete_item(*id));

        refetch_on_success(rename_item, items);
        refetch_on_success(delete_item, items);

        Self {
            patch,
            poster_upload,
            rename_item,
            delete_item,
        }
    }

    /// Wraps `rename_item`/`delete_item` as single-arg callbacks the
    /// playlist can hand to its per-row actions.
    fn rename_callback(self) -> Callback<(u64, String)> {
        Callback::new(move |(id, t): (u64, String)| {
            let _ = self.rename_item.dispatch((id, t));
        })
    }

    fn delete_callback(self) -> Callback<u64> {
        Callback::new(move |id: u64| {
            let _ = self.delete_item.dispatch(id);
        })
    }
}

// ─── Collection edit state ────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct CollectionEditState {
    title: RwSignal<String>,
    description: RwSignal<String>,
    poster: RwSignal<Option<String>>,
    commit_title: Callback<String>,
    commit_desc: Callback<String>,
}

impl CollectionEditState {
    fn new(collection: &Collection, patch: PatchAction) -> Self {
        let collection_id = collection.id;

        let title = RwSignal::new(collection.title.clone());
        let description = RwSignal::new(collection.description.clone().unwrap_or_default());
        let poster = RwSignal::new(collection.poster.clone());

        let commit_title = Callback::new(move |v: String| {
            title.set(v.clone());
            patch.dispatch((collection_id, "title".into(), Some(v)));
        });
        let commit_desc = Callback::new(move |v: String| {
            description.set(v.clone());
            let val = if v.is_empty() { None } else { Some(v) };
            patch.dispatch((collection_id, "description".into(), val));
        });

        Self {
            title,
            description,
            poster,
            commit_title,
            commit_desc,
        }
    }
}

// ─── Poster upload ────────────────────────────────────────────────────────

/// Builds the `Callback<File>` that `EditablePoster` expects, and installs
/// the `Effect` that copies the returned URL back into the `poster` signal.
fn use_poster_upload(
    section_slug: String,
    collection_id: u64,
    poster: RwSignal<Option<String>>,
    poster_upload: PosterUploadAction,
) -> Callback<web_sys::File> {
    let on_file = Callback::new(move |file: web_sys::File| {
        let fd = web_sys::FormData::new().unwrap();
        let _ = fd.append_with_str("section", &section_slug);
        let _ = fd.append_with_str("id", &collection_id.to_string());
        let _ = fd.append_with_blob_and_filename("poster_file", &file, &file.name());
        poster_upload.dispatch(fd);
    });

    Effect::new(move |_| {
        if let Some(Ok(url)) = poster_upload.value().get() {
            poster.set(Some(url));
        }
    });

    on_file
}

// ─── Placeholder & adapter builders ───────────────────────────────────────

fn make_poster_placeholder(is_audio: bool) -> ViewFn {
    ViewFn::from(move || {
        if is_audio {
            view! { <MusicPosterSvg/> }.into_any()
        } else {
            view! { <MoviePosterSvg/> }.into_any()
        }
    })
}

#[allow(clippy::too_many_arguments)]
fn make_playlist_adapter(
    section_slug: String,
    playlist_title: String,
    poster: Option<String>,
    is_audio: bool,
    is_series: bool,
    selected_season: RwSignal<Option<i64>>,
    initial_item_id: Option<u64>,
    on_rename: Callback<(u64, String)>,
    on_delete: Callback<u64>,
) -> impl Fn(Vec<Item>) -> PlaylistProps {
    move |list: Vec<Item>| PlaylistProps {
        items: list,
        audio: is_audio,
        is_series,
        selected_season,
        artwork: poster.clone(),
        playlist_title: playlist_title.clone(),
        section_slug: section_slug.clone(),
        initial_item_id,
        on_rename,
        on_delete,
    }
}

// ─── Page structure ───────────────────────────────────────────────────────

pub struct CollectionDetailPage {
    section: Resource<Result<Section, ServerFnError>>,
    collection: Resource<Result<Collection, ServerFnError>>,
    items: Resource<Result<Vec<Item>, ServerFnError>>,
    initial_item_id: Option<u64>,
}

#[lazy_route]
impl LazyRoute for CollectionDetailPage {
    fn data() -> Self {
        let slug = use_string_param("slug");
        let id = use_u64_param("id");
        let item_id_fn = use_optional_u64_param("item_id");

        let slug_a = slug;
        let slug_b = slug;

        Self {
            section: Resource::new(slug_a, fetch_section_by_slug),
            collection: Resource::new(
                move || (slug_b(), id()),
                |(s, i)| fetch_collection_detail(s, i),
            ),
            items: Resource::new(id, fetch_items),
            initial_item_id: item_id_fn(),
        }
    }

    fn view(this: Self) -> AnyView {
        let collection = this.collection;
        let items = this.items;
        let initial_item_id = this.initial_item_id;
        let adapter = move |section: Section| CollectionDetailBodyProps {
            section,
            collection,
            items,
            initial_item_id,
        };
        view! {
            <ResourceView resource=this.section view_fn=CollectionDetailBody adapter=adapter/>
        }
        .into_any()
    }
}

#[component]
fn CollectionDetailBody(
    section: Section,
    collection: Resource<Result<Collection, ServerFnError>>,
    items: Resource<Result<Vec<Item>, ServerFnError>>,
    initial_item_id: Option<u64>,
) -> impl IntoView {
    let adapter = move |c: Collection| CollectionContentProps {
        section: section.clone(),
        collection: c,
        items,
        initial_item_id,
    };
    view! {
        <ResourceView resource=collection view_fn=CollectionContent adapter=adapter/>
    }
}

#[component]
fn CollectionContent(
    section: Section,
    collection: Collection,
    items: Resource<Result<Vec<Item>, ServerFnError>>,
    initial_item_id: Option<u64>,
) -> impl IntoView {
    let collection_id = collection.id;
    let section_slug = section.slug.clone();
    let is_audio = matches!(section.media_kind(), MediaKind::Audio);
    let is_series = section.nested && !is_audio;

    let selected_season = RwSignal::new(None::<i64>);

    // Actions + edit state
    let actions = CollectionActions::new(items);
    let edit = CollectionEditState::new(&collection, actions.patch);

    // Refresh the items list when an upload finishes
    let upload = UploadJob::new();
    Effect::new(move |_| {
        if upload.done_tick.get() > 0 {
            items.refetch();
        }
    });

    // Poster upload wiring
    let on_poster_file = use_poster_upload(
        section_slug.clone(),
        collection_id,
        edit.poster,
        actions.poster_upload,
    );

    // Item edit/delete callbacks
    let on_rename = actions.rename_callback();
    let on_delete = actions.delete_callback();

    // Snapshots for the initial render + playlist adapter
    let poster_for_shell = edit.poster.get_untracked();
    let placeholder = make_poster_placeholder(is_audio);
    let icon = icon_for(section.media_kind());
    let badge_label = section.badge_label();

    let playlist_adapter = make_playlist_adapter(
        section_slug.clone(),
        edit.title.get_untracked(),
        edit.poster.get_untracked(),
        is_audio,
        is_series,
        selected_season,
        initial_item_id,
        on_rename,
        on_delete,
    );

    view! {
        <DetailShell poster=poster_for_shell>
            <DetailHero poster=view! {
                <EditablePoster
                    src=Signal::derive(move || edit.poster.get())
                    placeholder=placeholder
                    on_file=on_poster_file
                    input_id=format!("poster-collection-{collection_id}")
                />
            }>
                <HeroBadge label=badge_label.to_string() icon=icon/>
                <EditableText
                    value=Signal::derive(move || edit.title.get())
                    on_commit=edit.commit_title
                    class="text-3xl sm:text-4xl md:text-5xl font-black tracking-tight mb-2 text-white"
                />
                <HeroMeta>
                    <span class="flex items-center gap-1">
                        <ClockIcon/>
                        {move || format!(
                            "{} عنصر",
                            items.get()
                                .map(|r| r.map(|v| v.len()).unwrap_or(0))
                                .unwrap_or(0)
                        )}
                    </span>
                </HeroMeta>
                <div class="mt-4 max-w-2xl">
                    <EditableTextArea
                        value=Signal::derive(move || edit.description.get())
                        on_commit=edit.commit_desc
                        placeholder="أضف وصفاً..."
                        class="text-gray-300 leading-relaxed text-base sm:text-lg"
                    />
                </div>
                <AppendItems
                    upload=upload
                    section_slug=section_slug.clone()
                    collection_id=collection_id
                    is_audio=is_audio
                    is_series=is_series
                    selected_season=selected_season
                />
            </DetailHero>

            <div class="mt-10">
                <ResourceView
                    resource=items
                    view_fn=Playlist
                    adapter=playlist_adapter
                />
            </div>
        </DetailShell>
    }
}

// ─── Append items ─────────────────────────────────────────────────────────

fn accept_for_kind(is_audio: bool) -> &'static str {
    if is_audio {
        ".mp3,.m4a,.flac,.wav,.ogg,.opus,.aac"
    } else {
        ".mp4,.mkv,.mov,.webm,.avi,.m4v,.wmv,.flv,.ts"
    }
}

/// Pure: turns the selected files + metadata into the multipart body.
fn build_append_formdata(
    files: &web_sys::FileList,
    section_slug: &str,
    collection_id: u64,
    season: Option<i64>,
) -> web_sys::FormData {
    let fd = web_sys::FormData::new().unwrap();
    let _ = fd.append_with_str("section_slug", section_slug);
    let _ = fd.append_with_str("collection_id", &collection_id.to_string());

    if let Some(season) = season {
        let _ = fd.append_with_str("season_number", &season.to_string());
    }

    for i in 0..files.length() {
        if let Some(f) = files.get(i) {
            let file: web_sys::File = f.unchecked_into();
            let name = file.name();
            let stem = name.rsplitn(2, '.').last().unwrap_or(&name).to_string();
            let _ = fd.append_with_blob_and_filename(&format!("file_{i}"), &file, &name);
            let _ = fd.append_with_str(&format!("file_title_{i}"), &stem);
        }
    }
    fd
}

/// Wraps the `on:change` handler: reads the picked files, builds the
/// multipart body, dispatches the upload, clears the input.
fn make_file_input_handler(
    upload: UploadJob,
    section_slug: String,
    collection_id: u64,
    is_series: bool,
    selected_season: RwSignal<Option<i64>>,
) -> Callback<web_sys::Event> {
    Callback::new(move |ev: web_sys::Event| {
        let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        else {
            return;
        };
        let Some(files) = input.files() else { return };
        if files.length() == 0 {
            return;
        }

        let season = if is_series {
            selected_season.get_untracked()
        } else {
            None
        };

        let fd = build_append_formdata(&files, &section_slug, collection_id, season);
        upload.dispatch(fd);
        input.set_value("");
    })
}

fn upload_button_label_text(is_series: bool, season: Option<i64>) -> String {
    if is_series {
        let season_label = season
            .map(|s| format!(" إلى الموسم {s}"))
            .unwrap_or_default();
        format!("إضافة حلقات{season_label}")
    } else {
        "إضافة ملفات".to_string()
    }
}

#[component]
fn UploadErrorBanner(error: Signal<Option<String>>) -> impl IntoView {
    view! {
        {move || error.get().map(|e| view! {
            <div class="mt-3 bg-red-500/15 text-red-300 border border-red-500/30 \
                        rounded-xl p-3 text-sm">
                {e}
            </div>
        })}
    }
}

#[component]
fn AppendItems(
    upload: UploadJob,
    #[prop(into)] section_slug: String,
    collection_id: u64,
    is_audio: bool,
    is_series: bool,
    selected_season: RwSignal<Option<i64>>,
) -> impl IntoView {
    let input_id = format!("append-input-{collection_id}");
    let input_id: &'static str = input_id.leak();

    let upload_pending = upload.pending;
    let upload_status = upload.status;
    let upload_error = upload.error();

    let accept = accept_for_kind(is_audio);
    let on_files = make_file_input_handler(
        upload,
        section_slug,
        collection_id,
        is_series,
        selected_season,
    );

    let edit_on = use_edit_mode();

    view! {
        <Show when=move || edit_on.get()>
            <div class="mt-6 flex items-center gap-3 flex-wrap">
                <input
                    type="file"
                    id=input_id
                    class="hidden"
                    multiple
                    accept=accept
                    on:change=move |ev| on_files.run(ev)
                />
                <label
                    for=input_id
                    class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl \
                           bg-green-500/20 hover:bg-green-500/30 text-green-300 \
                           text-sm font-medium cursor-pointer transition"
                >
                    <UploadIcon/>
                    {move || upload_button_label_text(is_series, selected_season.get())}
                </label>
                <Show when=move || upload_pending.get()>
                    <span class="text-cyan-300 text-sm">"جاري الرفع..."</span>
                </Show>
            </div>
            <div class="mt-3">
                <UploadProgress status=Signal::derive(move || upload_status.get())/>
            </div>
            <UploadErrorBanner error=upload_error/>
        </Show>
    }
}

// ─── Playlist ─────────────────────────────────────────────────────────────

#[component]
fn Playlist(
    items: Vec<Item>,
    audio: bool,
    is_series: bool,
    selected_season: RwSignal<Option<i64>>,
    artwork: Option<String>,
    playlist_title: String,
    section_slug: String,
    initial_item_id: Option<u64>,
    on_rename: Callback<(u64, String)>,
    on_delete: Callback<u64>,
) -> impl IntoView {
    let all_seasons: Vec<i64> = items
        .iter()
        .filter_map(|i| i.season_number)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    if selected_season.get_untracked().is_none() {
        let initial = initial_item_id
            .and_then(|target| items.iter().find(|it| it.id == target))
            .and_then(|it| it.season_number)
            .or_else(|| all_seasons.first().copied())
            .or(if is_series { Some(1) } else { None });
        selected_season.set(initial);
    }

    let items_for_body = items.clone();
    let body = move || {
        let current = selected_season.get();

        let filtered: Vec<Item> = if is_series {
            items_for_body
                .iter()
                .filter(|it| it.season_number == current)
                .cloned()
                .collect()
        } else {
            items_for_body.clone()
        };

        if filtered.is_empty() {
            let msg = if is_series {
                "لا توجد حلقات في هذا الموسم بعد. اضغط «تعديل» ثم «إضافة حلقات»."
            } else {
                "لا توجد ملفات بعد. اضغط «تعديل» ثم «إضافة ملفات»."
            };
            return Either::Left(view! {
                <div class="py-12 text-center text-gray-500 text-sm">{msg}</div>
            });
        }

        let initial_index = initial_item_id
            .and_then(|target| filtered.iter().position(|it| it.id == target))
            .unwrap_or(0);

        let media_items: Vec<MediaItem> = filtered
            .iter()
            .map(|it| {
                let mut mi = MediaItem::new(it.id, it.display_title(), it.file_path());
                if let Some(season) = it.season_number {
                    mi = mi.with_subtitle(format!("S{season:02}"));
                }
                mi
            })
            .collect();

        Either::Right(view! {
            <MediaPlayer
                items=media_items.into()
                initial_index=initial_index
                audio=audio
                artwork=artwork.clone()
                playlist_title=playlist_title.clone()
                on_rename=on_rename
                on_delete=on_delete
                show_download=true
            />
        })
    };

    let selector = is_series.then(|| {
        view! { <SeasonBar seasons=all_seasons.clone() selected_season=selected_season/> }
    });

    let _ = section_slug;

    view! {
        {selector}
        {body}
    }
}

// ─── Season bar ───────────────────────────────────────────────────────────

fn next_season_number(seasons: &[i64]) -> i64 {
    seasons.iter().copied().max().unwrap_or(0) + 1
}

fn season_options_with_current(seasons: &[i64], current: Option<i64>) -> Vec<i64> {
    let mut list = seasons.to_vec();
    if let Some(cur) = current
        && !list.contains(&cur)
    {
        list.push(cur);
        list.sort();
    }
    if list.is_empty() {
        list.push(1);
    }
    list
}

#[component]
fn SeasonBar(seasons: Vec<i64>, selected_season: RwSignal<Option<i64>>) -> impl IntoView {
    let edit_on = use_edit_mode();

    let add_season = {
        let seasons = seasons.clone();
        move |_| selected_season.set(Some(next_season_number(&seasons)))
    };

    let options = {
        let list = seasons.clone();
        move || season_options_with_current(&list, selected_season.get())
    };

    let add_button = move || {
        edit_on.get().then(|| {
            view! {
                <button
                    type="button"
                    on:click=add_season.clone()
                    class="px-3 py-1.5 rounded-xl bg-white/10 hover:bg-white/20 \
                           text-white text-sm transition"
                    aria-label="إضافة موسم جديد"
                >
                    "+ موسم"
                </button>
            }
        })
    };

    view! {
        <div class="flex items-center gap-2 mb-4 flex-wrap">
            <span class="text-gray-300 text-sm">"الموسم:"</span>
            <select
                class="bg-white/10 backdrop-blur-md text-white rounded-xl py-1.5 px-3 \
                       focus:outline-none focus:ring-1 focus:ring-cyan-400"
                prop:value=move || {
                    selected_season.get().map(|v| v.to_string()).unwrap_or_default()
                }
                on:change=move |ev| {
                    if let Some(sel) = ev
                        .target()
                        .and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok())
                        && let Ok(v) = sel.value().parse::<i64>()
                    {
                        selected_season.set(Some(v));
                    }
                }
            >
                <For each=options key=|s| *s let:season>
                    <option value=season.to_string()>
                        {format!("الموسم {}", season)}
                    </option>
                </For>
            </select>
            {add_button}
        </div>
    }
}
