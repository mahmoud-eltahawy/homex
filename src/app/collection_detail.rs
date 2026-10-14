use std::collections::BTreeSet;
use std::sync::Arc;

use leptos::{either::Either, prelude::*};
use leptos_router::{LazyRoute, hooks::use_navigate, lazy_route};
use uuid::Uuid;
use web_sys::wasm_bindgen::JsCast;

use leptos_media_player::{MediaItem, WatchPage};

use crate::app::{
    collections::{
        delete_item, fetch_collection_detail, fetch_collection_summaries, fetch_items,
        move_collection_to_section, move_item, move_items_to_collection, patch_collection_field,
        patch_collection_tags, patch_item_title, regenerate_collection_poster,
        upload_collection_poster,
    },
    common::refetch_on_success,
    detail::{DetailHero, DetailShell, DownloadButton, HeroBadge, HeroMeta},
    error::AppError,
    error_view::{ErrorStyle, ErrorView, action_error},
    icons::{ClockIcon, MoviePosterSvg, MusicPosterSvg, UploadIcon, icon_for},
    inline_edit::{EditablePoster, EditableText, EditableTextArea, use_edit_mode},
    model::{Collection, CollectionSummary, Item, MediaKind, Section},
    resource_view::ResourceView,
    route_params::{use_optional_uuid_param, use_string_param, use_uuid_param},
    sections::fetch_section_by_slug,
    tags::TagEditor,
    unclassified::move_items_to_incoming,
    upload_job::{UploadJob, UploadProgress},
};

// ─── Multipart form field names (client-side) ─────────────────────────────

mod fields;

// ─── Action type aliases ──────────────────────────────────────────────────

type PatchAction = Action<(Uuid, String, Option<String>), Result<(), AppError>>;
type PosterUploadAction = Action<web_sys::FormData, Result<String, AppError>>;
type RegenerateAction = Action<(String, Uuid), Result<String, AppError>>;
type RenameItemAction = Action<(Uuid, String), Result<(), AppError>>;
type DeleteItemAction = Action<Uuid, Result<(), AppError>>;
type TagsAction = Action<(Uuid, Vec<String>), Result<(), AppError>>;
type MoveToIncomingAction = Action<Vec<Uuid>, Result<(), AppError>>;

// ─── Collection actions bundle ────────────────────────────────────────────

#[derive(Clone, Copy)]
struct CollectionActions {
    patch: PatchAction,
    poster_upload: PosterUploadAction,
    regenerate: RegenerateAction,
    rename_item: RenameItemAction,
    delete_item: DeleteItemAction,
    move_item: Action<(Uuid, bool), Result<(), AppError>>,
    tags: TagsAction,
}

impl CollectionActions {
    fn new(items: Resource<Result<Vec<Item>, AppError>>) -> Self {
        let patch = Action::new_local(|(id, field, value): &(Uuid, String, Option<String>)| {
            patch_collection_field(*id, field.clone(), value.clone())
        });
        let poster_upload =
            Action::new_local(|fd: &web_sys::FormData| upload_collection_poster(fd.clone().into()));
        let regenerate = Action::new_local(|(slug, id): &(String, Uuid)| {
            regenerate_collection_poster(slug.clone(), *id)
        });
        let rename_item =
            Action::new_local(|(id, t): &(Uuid, String)| patch_item_title(*id, t.clone()));
        let delete_item = Action::new_local(|id: &Uuid| delete_item(*id));
        let tags = Action::new_local(|(id, t): &(Uuid, Vec<String>)| {
            patch_collection_tags(*id, t.clone())
        });

        refetch_on_success(rename_item, items);
        refetch_on_success(delete_item, items);

        let move_item = Action::new_local(|(id, up): &(Uuid, bool)| move_item(*id, *up));
        refetch_on_success(move_item, items);

        Self {
            patch,
            poster_upload,
            regenerate,
            rename_item,
            delete_item,
            move_item,
            tags,
        }
    }

    fn rename_callback(self) -> Callback<(Uuid, String)> {
        Callback::new(move |(id, t): (Uuid, String)| {
            let _ = self.rename_item.dispatch((id, t));
        })
    }

    fn delete_callback(self) -> Callback<Uuid> {
        Callback::new(move |id: Uuid| {
            let _ = self.delete_item.dispatch(id);
        })
    }

    fn move_callback(self) -> Callback<(Uuid, bool)> {
        Callback::new(move |(id, up): (Uuid, bool)| {
            let _ = self.move_item.dispatch((id, up));
        })
    }

    fn patch_error(self) -> Signal<Option<AppError>> {
        action_error(self.patch)
    }
    fn poster_error(self) -> Signal<Option<AppError>> {
        action_error(self.poster_upload)
    }
    fn regenerate_error(self) -> Signal<Option<AppError>> {
        action_error(self.regenerate)
    }
    fn rename_error(self) -> Signal<Option<AppError>> {
        action_error(self.rename_item)
    }
    fn delete_error(self) -> Signal<Option<AppError>> {
        action_error(self.delete_item)
    }
    fn move_error(self) -> Signal<Option<AppError>> {
        action_error(self.move_item)
    }
    fn tags_error(self) -> Signal<Option<AppError>> {
        action_error(self.tags)
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

// ─── Poster upload (audio only) ───────────────────────────────────────────

fn use_poster_upload(
    section_slug: String,
    collection_id: Uuid,
    poster: RwSignal<Option<String>>,
    poster_upload: PosterUploadAction,
) -> Callback<web_sys::File> {
    let on_file = Callback::new(move |file: web_sys::File| {
        let fd = web_sys::FormData::new().unwrap();
        let _ = fd.append_with_str(fields::SECTION, &section_slug);
        let _ = fd.append_with_str(fields::ID, &collection_id.to_string());
        let _ = fd.append_with_blob_and_filename(fields::POSTER_FILE, &file, &file.name());
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
    playlist_title: String,
    poster: Option<String>,
    is_audio: bool,
    is_series: bool,
    selected_season: RwSignal<Option<i64>>,
    initial_item_id: Option<Uuid>,
    on_rename: Callback<(Uuid, String)>,
    on_delete: Callback<Uuid>,
    on_move: Callback<(Uuid, bool)>,
) -> impl Fn(Vec<Item>) -> PlaylistProps {
    move |list: Vec<Item>| PlaylistProps {
        items: list,
        audio: is_audio,
        is_series,
        selected_season,
        artwork: poster.clone(),
        playlist_title: playlist_title.clone(),
        initial_item_id,
        on_rename,
        on_delete,
        on_move,
    }
}

// ─── Page structure ───────────────────────────────────────────────────────

pub struct CollectionDetailPage {
    section: Resource<Result<Section, AppError>>,
    collection: Resource<Result<Collection, AppError>>,
    items: Resource<Result<Vec<Item>, AppError>>,
    initial_item_id: Option<Uuid>,
}

#[lazy_route]
impl LazyRoute for CollectionDetailPage {
    fn data() -> Self {
        let slug = use_string_param("slug");
        let id = use_uuid_param("id");
        let item_id_fn = use_optional_uuid_param("item_id");

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
    collection: Resource<Result<Collection, AppError>>,
    items: Resource<Result<Vec<Item>, AppError>>,
    initial_item_id: Option<Uuid>,
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
    items: Resource<Result<Vec<Item>, AppError>>,
    initial_item_id: Option<Uuid>,
) -> impl IntoView {
    let collection_id = collection.id;
    let section_slug = section.slug.clone();
    let is_audio = matches!(section.media_kind(), MediaKind::Audio);
    let is_series = section.nested && !is_audio;

    let selected_season = RwSignal::new(None::<i64>);
    let edit_on = use_edit_mode();
    let organizing = RwSignal::new(false);

    let actions = CollectionActions::new(items);
    let edit = CollectionEditState::new(&collection, actions.patch);

    let tags_signal = RwSignal::new(collection.tags.clone());
    let on_tags_commit = Callback::new(move |names: Vec<String>| {
        let _ = actions.tags.dispatch((collection_id, names));
    });

    let upload = UploadJob::new();
    Effect::new(move |_| {
        if upload.done_tick.get() > 0 {
            items.refetch();
        }
    });

    let slug_for_poster = section_slug.clone();
    let slug_for_regen = section_slug.clone();
    let slug_for_picker = section_slug.clone();
    let slug_for_append = section_slug;

    // Audio: manual picker.
    let on_poster_file = use_poster_upload(
        slug_for_poster,
        collection_id,
        edit.poster,
        actions.poster_upload,
    );

    // Video: regenerate.
    let on_regenerate = Callback::new(move |_: ()| {
        actions
            .regenerate
            .dispatch((slug_for_regen.clone(), collection_id));
    });
    Effect::new(move |_| {
        if let Some(Ok(url)) = actions.regenerate.value().get() {
            edit.poster.set(Some(url));
        }
    });
    let regenerating = actions.regenerate.pending();

    let audio_picker = is_audio.then_some(on_poster_file);
    let video_regen = (!is_audio).then_some(on_regenerate);

    let on_rename = actions.rename_callback();
    let on_delete = actions.delete_callback();
    let on_move = actions.move_callback();

    let poster_for_shell = edit.poster.get_untracked();
    let placeholder = make_poster_placeholder(is_audio);
    let icon = icon_for(section.media_kind(), section.nested);
    let badge_label = section.badge_label();
    let kind = section.media_kind();

    let patch_error = actions.patch_error();
    let poster_error = actions.poster_error();
    let regenerate_error = actions.regenerate_error();
    let tags_error = actions.tags_error();
    let rename_error = actions.rename_error();
    let delete_error = actions.delete_error();
    let move_error = actions.move_error();

    let playlist_adapter = Arc::new(make_playlist_adapter(
        edit.title.get_untracked(),
        edit.poster.get_untracked(),
        is_audio,
        is_series,
        selected_season,
        initial_item_id,
        on_rename,
        on_delete,
        on_move,
    ));

    view! {
        <DetailShell poster=poster_for_shell>
            <DetailHero poster=view! {
                <EditablePoster
                    src=Signal::derive(move || edit.poster.get())
                    placeholder=placeholder
                    input_id=format!("poster-collection-{collection_id}")
                    on_file=audio_picker
                    on_regenerate=video_regen
                    regenerating=Signal::derive(move || regenerating.get())
                />
            }>
                <HeroBadge label=badge_label.to_string() icon=icon/>
                <Show when=move || edit_on.get()>
                    <div class="mb-3 flex items-center gap-2 flex-wrap">
                        <MoveCollectionPicker
                            collection_id=collection_id
                            current_slug=slug_for_picker.clone()
                            media_kind=kind
                        />
                        <button
                            type="button"
                            on:click=move |_| organizing.update(|o| *o = !*o)
                            class=move || format!(
                                "text-xs px-3 py-1 rounded-lg border transition \
                                 {}",
                                if organizing.get() {
                                    "bg-cyan-500/20 border-cyan-400/50 text-white"
                                } else {
                                    "bg-white/5 border-white/10 text-gray-300 hover:bg-white/10 hover:text-white"
                                }
                            )
                        >
                            {move || if organizing.get() { "Done organizing" } else { "Organize items" }}
                        </button>
                    </div>
                </Show>
                <EditableText
                    value=Signal::derive(move || edit.title.get())
                    on_commit=edit.commit_title
                    class="text-3xl sm:text-4xl md:text-5xl font-black tracking-tight mb-2 text-white"
                />
                <HeroMeta>
                    <span class="flex items-center gap-1">
                        <ClockIcon/>
                        {move || format!(
                            "{} items",
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
                        placeholder="Add a description..."
                        class="text-gray-300 leading-relaxed text-base sm:text-lg"
                    />
                </div>
                <div class="mt-4 max-w-2xl">
                    <TagEditor tags=tags_signal on_commit=on_tags_commit/>
                </div>
                <div class="mt-2 max-w-2xl space-y-1">
                    <ErrorView error=patch_error/>
                    <ErrorView error=tags_error/>
                    <ErrorView error=poster_error/>
                    <ErrorView error=regenerate_error/>
                </div>
                <AppendItems
                    upload=upload
                    section_slug=slug_for_append
                    collection_id=collection_id
                    is_audio=is_audio
                    is_series=is_series
                    selected_season=selected_season
                />
            </DetailHero>

            <div class="mt-10">
                {{
                    let adapter_source = Arc::clone(&playlist_adapter);
                    move || {
                        if organizing.get() {
                            Either::Left(
                                view! {
                                    <OrganizeItemsPanel
                                        items=items
                                        current_collection_id=collection_id
                                        media_kind=kind
                                        on_close=Callback::new(move |_| organizing.set(false))
                                    />
                                }
                                .into_any(),
                            )
                        } else {
                            let adapter = Arc::clone(&adapter_source);
                            let adapter_fn = move |v: Vec<Item>| adapter(v);
                            Either::Right(
                                view! {
                                    <ResourceView
                                        resource=items
                                        view_fn=Playlist
                                        adapter=adapter_fn
                                    />
                                }
                                .into_any(),
                            )
                        }
                    }
                }}
            </div>

            <div class="mt-4 space-y-1">
                <ErrorView error=rename_error/>
                <ErrorView error=delete_error/>
                <ErrorView error=move_error/>
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

fn build_append_formdata(
    files: &web_sys::FileList,
    section_slug: &str,
    collection_id: Uuid,
    season: Option<i64>,
) -> web_sys::FormData {
    let fd = web_sys::FormData::new().unwrap();
    let _ = fd.append_with_str(fields::SECTION_SLUG, section_slug);
    let _ = fd.append_with_str(fields::COLLECTION_ID, &collection_id.to_string());

    if let Some(season) = season {
        let _ = fd.append_with_str(fields::SEASON_NUMBER, &season.to_string());
    }

    for i in 0..files.length() {
        if let Some(f) = files.get(i) {
            let file: web_sys::File = f.unchecked_into();
            let name = file.name();
            let stem = name.rsplitn(2, '.').last().unwrap_or(&name).to_string();
            let _ = fd.append_with_blob_and_filename(&fields::file(i as usize), &file, &name);
            let _ = fd.append_with_str(&fields::file_title(i as usize), &stem);
        }
    }
    fd
}

fn make_file_input_handler(
    upload: UploadJob,
    section_slug: String,
    collection_id: Uuid,
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
            .map(|s| format!(" to season {s}"))
            .unwrap_or_default();
        format!("Add episodes{season_label}")
    } else {
        "Add files".to_string()
    }
}

#[component]
fn AppendItems(
    upload: UploadJob,
    #[prop(into)] section_slug: String,
    collection_id: Uuid,
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
                    <span class="text-cyan-300 text-sm">"Uploading..."</span>
                </Show>
            </div>
            <div class="mt-3">
                <UploadProgress status=Signal::derive(move || upload_status.get())/>
            </div>
            <div class="mt-3">
                <ErrorView error=upload_error style=ErrorStyle::Banner/>
            </div>
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
    initial_item_id: Option<Uuid>,
    on_rename: Callback<(Uuid, String)>,
    on_delete: Callback<Uuid>,
    on_move: Callback<(Uuid, bool)>,
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
                "No episodes in this season yet. Click \"Edit\" then \"Add episodes\"."
            } else {
                "No files yet. Click \"Edit\" then \"Add files\"."
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

        // `WatchPage` collapses the playlist panel — and its download control
        // — when there is a single entry. Render a download button ourselves
        // in that case so the file is still reachable.
        let single_item_download = if filtered.len() == 1 {
            let item = &filtered[0];
            Some((item.file_path(), item.display_title()))
        } else {
            None
        };

        Either::Right(view! {
            <>
                {single_item_download.map(|(href, name)| view! {
                    <div class="mb-4 flex justify-end">
                        <DownloadButton href=href download_name=name/>
                    </div>
                })}
                <WatchPage
                    edit_mode=use_edit_mode()
                    items=media_items.into()
                    initial_idx=initial_index
                    audio=audio
                    artwork=artwork.clone()
                    playlist_title=playlist_title.clone()
                    on_rename=on_rename
                    on_delete=on_delete
                    on_move=on_move
                    show_download=true
                />
            </>
        })
    };

    let selector = is_series.then(|| {
        view! { <SeasonBar seasons=all_seasons.clone() selected_season=selected_season/> }
    });

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
                    aria-label="Add new season"
                >
                    "+ Season"
                </button>
            }
        })
    };

    view! {
        <div class="flex items-center gap-2 mb-4 flex-wrap">
            <span class="text-gray-300 text-sm">"Season:"</span>
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
                        {format!("Season {}", season)}
                    </option>
                </For>
            </select>
            {add_button}
        </div>
    }
}

// ─── Move collection to another section ───────────────────────────────────

#[component]
fn MoveCollectionPicker(
    collection_id: Uuid,
    current_slug: String,
    media_kind: MediaKind,
) -> impl IntoView {
    let sections: Resource<Result<Vec<Section>, AppError>> = expect_context();
    let navigate = use_navigate();

    let action =
        Action::new_local(|(id, to): &(Uuid, String)| move_collection_to_section(*id, to.clone()));

    let destination = RwSignal::new(String::new());

    let current_for_options = current_slug.clone();
    let options: Signal<Vec<Section>> = Signal::derive(move || {
        sections
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
            .into_iter()
            .filter(|s| s.media_kind() == media_kind && s.slug != current_for_options)
            .collect()
    });

    let current_for_change = current_slug;
    let on_change = move |ev: web_sys::Event| {
        let to = event_target_value(&ev);
        if to.is_empty() || to == current_for_change {
            return;
        }
        destination.set(to.clone());
        action.dispatch((collection_id, to));
    };

    Effect::new(move |_| {
        if matches!(action.value().get(), Some(Ok(()))) {
            let to = destination.get_untracked();
            if !to.is_empty() {
                navigate(&format!("/s/{to}"), Default::default());
            }
        }
    });

    view! {
        <Show when=move || !options.get().is_empty()>
            <div class="flex items-center gap-2 flex-wrap">
                <select
                    on:change=on_change.clone()
                    disabled=move || action.pending().get()
                    class="bg-white/10 backdrop-blur-md text-white text-xs rounded-lg \
                           py-1 px-2 focus:outline-none focus:ring-1 focus:ring-cyan-400 \
                           disabled:opacity-50"
                >
                    <option value="">"Move to…"</option>
                    <For each=move || options.get() key=|s| s.slug.clone() let:s>
                        <option value=s.slug.clone()>{s.title.clone()}</option>
                    </For>
                </select>
                <ErrorView error=action_error(action)/>
            </div>
        </Show>
    }
}

// ─── Organize items panel ─────────────────────────────────────────────────

#[component]
fn OrganizeItemsPanel(
    items: Resource<Result<Vec<Item>, AppError>>,
    current_collection_id: Uuid,
    media_kind: MediaKind,
    on_close: Callback<()>,
) -> impl IntoView {
    let kind_str = media_kind.as_str().to_string();
    let destinations: Resource<Result<Vec<CollectionSummary>, AppError>> =
        Resource::new(move || kind_str.clone(), fetch_collection_summaries);

    let selected: RwSignal<Vec<Uuid>> = RwSignal::new(Vec::new());
    let destination_id: RwSignal<Option<Uuid>> = RwSignal::new(None);

    let move_action = Action::new_local(|(ids, to): &(Vec<Uuid>, Uuid)| {
        move_items_to_collection(ids.clone(), *to)
    });

    Effect::new(move |_| {
        if matches!(move_action.value().get(), Some(Ok(()))) {
            items.refetch();
            selected.set(Vec::new());
            destination_id.set(None);
            on_close.run(());
        }
    });

    let to_incoming: MoveToIncomingAction =
        Action::new_local(|ids: &Vec<Uuid>| move_items_to_incoming(ids.clone()));

    Effect::new(move |_| {
        if matches!(to_incoming.value().get(), Some(Ok(()))) {
            items.refetch();
            selected.set(Vec::new());
            destination_id.set(None);
            on_close.run(());
        }
    });

    let toggle_all = Callback::new(move |_: ()| {
        if !selected.get_untracked().is_empty() {
            selected.set(Vec::new());
            return;
        }
        if let Some(Ok(list)) = items.get_untracked() {
            selected.set(list.into_iter().map(|i| i.id).collect());
        }
    });

    let on_submit = move |_| {
        let ids = selected.get_untracked();
        let Some(to) = destination_id.get_untracked() else {
            return;
        };
        if ids.is_empty() {
            return;
        }
        move_action.dispatch((ids, to));
    };

    view! {
        <div class="rounded-2xl border border-white/10 bg-white/[0.02] p-4 md:p-6">
            <div class="flex items-center justify-between mb-4">
                <h3 class="text-lg font-bold text-white">"Move items to another collection"</h3>
                <button
                    type="button"
                    on:click=move |_| on_close.run(())
                    class="text-gray-400 hover:text-white transition text-sm"
                >
                    "Close"
                </button>
            </div>

            <div class="flex items-center gap-3 mb-3 text-sm">
                <button
                    type="button"
                    on:click=move |_| toggle_all.run(())
                    class="px-2 py-1 rounded-lg bg-white/5 hover:bg-white/10 text-gray-300 transition"
                >
                    {move || if selected.get().is_empty() { "Select all" } else { "Clear" }}
                </button>
                <span class="text-gray-500">
                    {move || format!("{} selected", selected.get().len())}
                </span>
            </div>

            {
                move || match items.get() {
                    None => view! {
                        <div class="py-8 text-center text-gray-500 text-sm">"Loading…"</div>
                    }.into_any(),
                    Some(Err(e)) => view! {
                        <div class="py-8 text-center text-red-300 text-sm">
                            {e.message}
                        </div>
                    }.into_any(),
                    Some(Ok(list)) if list.is_empty() => view! {
                        <div class="py-8 text-center text-gray-500 text-sm">
                            "No items to move."
                        </div>
                    }.into_any(),
                    Some(Ok(list)) => view! {
                        <div class="space-y-px max-h-[400px] overflow-auto rounded-lg border border-white/5">
                            <For each=move || list.clone() key=|i| i.id let:item>
                                {
                                    let id = item.id;
                                    let label = item.display_title();
                                    let season = item.season_number;
                                    view! {
                                        <label class="flex items-center gap-3 px-3 py-2 hover:bg-white/5 cursor-pointer">
                                            <input
                                                type="checkbox"
                                                prop:checked=move || selected.get().contains(&id)
                                                on:change=move |_| {
                                                    selected.update(|ids| {
                                                        if let Some(pos) = ids.iter().position(|x| *x == id) {
                                                            ids.remove(pos);
                                                        } else {
                                                            ids.push(id);
                                                        }
                                                    });
                                                }
                                                class="accent-cyan-400 w-4 h-4 shrink-0"
                                            />
                                            <span class="text-sm text-white truncate flex-1">
                                                {label}
                                            </span>
                                            {season.map(|s| view! {
                                                <span class="text-[10px] font-mono text-gray-500 shrink-0">
                                                    {format!("S{s:02}")}
                                                </span>
                                            })}
                                        </label>
                                    }
                                }
                            </For>
                        </div>
                    }.into_any(),
                }
            }

            <div class="mt-4 flex items-center gap-3 flex-wrap">
                <select
                    on:change=move |ev| {
                        let v = event_target_value(&ev);
                        destination_id.set(Uuid::parse_str(&v).ok());
                    }
                    class="bg-white/10 text-white rounded-lg px-3 py-1.5 text-sm \
                           focus:outline-none focus:ring-1 focus:ring-cyan-400 min-w-64"
                >
                    <option value="">"Choose destination…"</option>
                    {
                        move || destinations
                            .get()
                            .and_then(|r| r.ok())
                            .unwrap_or_default()
                            .into_iter()
                            .filter(|c| c.id != current_collection_id)
                            .map(|c| view! {
                                <option value=c.id.to_string()>
                                    {format!("{} — {}", c.section_title, c.title)}
                                </option>
                            })
                            .collect_view()
                    }
                </select>

                <button
                    type="button"
                    on:click=on_submit
                    disabled=move || {
                        selected.get().is_empty()
                            || destination_id.get().is_none()
                            || move_action.pending().get()
                    }
                    class="px-4 py-1.5 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 \
                           hover:from-cyan-400 hover:to-blue-400 text-white font-bold text-sm \
                           disabled:opacity-50 transition"
                >
                    {move || {
                        let n = selected.get().len();
                        if move_action.pending().get() {
                            "Moving…".to_string()
                        } else if n == 0 {
                            "Move items".to_string()
                        } else {
                            format!("Move {n} item{}", if n == 1 { "" } else { "s" })
                        }
                    }}
                </button>

                <button
                    type="button"
                    on:click=move |_| {
                        let ids = selected.get_untracked();
                        if !ids.is_empty() {
                            to_incoming.dispatch(ids);
                        }
                    }
                    disabled=move || {
                        selected.get().is_empty() || to_incoming.pending().get()
                    }
                    class="px-4 py-1.5 rounded-lg bg-white/10 hover:bg-white/20 \
                           text-white text-sm font-medium transition \
                           disabled:opacity-50"
                >
                    {move || {
                        let n = selected.get().len();
                        if to_incoming.pending().get() {
                            "Returning…".to_string()
                        } else if n == 0 {
                            "Move to incoming".to_string()
                        } else {
                            format!("Move {n} to incoming")
                        }
                    }}
                </button>

                <ErrorView error=action_error(move_action)/>
                <ErrorView error=action_error(to_incoming)/>
            </div>
        </div>
    }
}
