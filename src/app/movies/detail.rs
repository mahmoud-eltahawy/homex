use crate::app::{
    detail::{DetailHero, DetailShell, HeroBadge},
    icons::{DeleteIcon, MovieIcon, MoviePosterSvg, UploadIcon},
    inline_edit::{EditablePoster, EditableText, EditableTextArea},
    media_api::{patch_field, upload_poster_inline},
    media_player::{MediaItem, MediaPlayer},
    model::{Movie, MovieChapter},
    resource_view::ResourceView,
    route_params::use_u64_param,
};
use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};
use web_sys::wasm_bindgen::JsCast;

#[server]
pub async fn fetch_movie_detail(id: u64) -> Result<crate::app::model::Movie, ServerFnError> {
    use crate::app::model::{MediaFile, Movie, MovieChapter};
    use crate::app::server::AppState;

    let state: AppState = expect_context();

    #[derive(sqlx::FromRow)]
    struct Row {
        id: i64,
        title: String,
        poster: Option<String>,
        description: Option<String>,
    }

    let m: Row = sqlx::query_as("SELECT id, title, poster, description FROM movies WHERE id = ?")
        .bind(id as i64)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("not found"))?;

    #[derive(sqlx::FromRow)]
    struct ChRow {
        id: i64,
        number: i64,
        title: Option<String>,
        poster: Option<String>,
        description: Option<String>,
        file_id: i64,
        size: i64,
        dur: i64,
    }
    let chapters: Vec<ChRow> = sqlx::query_as(
        "SELECT mc.id, mc.number, mc.title, mc.poster, mc.description, \
                f.id AS file_id, f.size_bytes AS size, \
                CAST(f.duration_secs AS INTEGER) AS dur \
         FROM movie_chapters mc JOIN files f ON f.id = mc.file_id \
         WHERE mc.movie_id = ? ORDER BY mc.number",
    )
    .bind(id as i64)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(Movie {
        id: m.id as u64,
        title: m.title,
        poster: m.poster,
        description: m.description,
        chapters: chapters
            .into_iter()
            .map(|c| MovieChapter {
                id: c.id as u64,
                number: c.number as u8,
                title: c.title,
                poster: c.poster,
                description: c.description,
                file: MediaFile {
                    id: c.file_id as u64,
                    path: format!("/media/{}", c.file_id),
                    size: c.size as u64,
                    duration: c.dur as u64,
                },
            })
            .collect(),
    })
}

#[server]
async fn fetch_movie_chapters(id: u64) -> Result<Vec<MovieChapter>, ServerFnError> {
    use crate::app::model::{MediaFile, MovieChapter};
    use crate::app::server::AppState;

    let state: AppState = expect_context();

    #[derive(sqlx::FromRow)]
    struct ChRow {
        id: i64,
        number: i64,
        title: Option<String>,
        poster: Option<String>,
        description: Option<String>,
        file_id: i64,
        size: i64,
        dur: i64,
    }

    let rows: Vec<ChRow> = sqlx::query_as(
        "SELECT mc.id, mc.number, mc.title, mc.poster, mc.description, \
                f.id AS file_id, f.size_bytes AS size, \
                CAST(f.duration_secs AS INTEGER) AS dur \
         FROM movie_chapters mc JOIN files f ON f.id = mc.file_id \
         WHERE mc.movie_id = ? ORDER BY mc.number",
    )
    .bind(id as i64)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|c| MovieChapter {
            id: c.id as u64,
            number: c.number as u8,
            title: c.title,
            poster: c.poster,
            description: c.description,
            file: MediaFile {
                id: c.file_id as u64,
                path: format!("/media/{}", c.file_id),
                size: c.size as u64,
                duration: c.dur as u64,
            },
        })
        .collect())
}

pub struct MovieDetailPage {
    movie: Resource<Result<Movie, ServerFnError>>,
}

#[lazy_route]
impl LazyRoute for MovieDetailPage {
    fn data() -> Self {
        let movie = Resource::new(use_u64_param("id"), fetch_movie_detail);
        Self { movie }
    }
    fn view(this: Self) -> AnyView {
        let adapter = |movie| MovieDetailProps { movie };
        view! { <ResourceView resource=this.movie view_fn=MovieDetail adapter=adapter /> }
            .into_any()
    }
}

// ─── MovieDetail ────────────────────────────────────────────────────────────

#[component]
fn MovieDetail(movie: Movie) -> impl IntoView {
    let id = movie.id;

    let title = RwSignal::new(movie.title.clone());
    let description = RwSignal::new(movie.description.clone().unwrap_or_default());
    let poster = RwSignal::new(movie.poster.clone());

    // Bumped to trigger a chapters refetch after an upload or delete.
    let refresh = RwSignal::new(0u32);
    let chapters = Resource::new(
        move || (id, refresh.get()),
        |(id, _)| fetch_movie_chapters(id),
    );

    // ── Server actions ──────────────────────────────────────────────────
    let patch = Action::new_local(
        |(kind, id, field, value): &(String, u64, String, Option<String>)| {
            patch_field(kind.clone(), *id, field.clone(), value.clone())
        },
    );

    let poster_upload =
        Action::new_local(|fd: &web_sys::FormData| upload_poster_inline(fd.clone().into()));

    let delete_child = Action::new_local(|(kind, id): &(String, u64)| {
        crate::app::media_api::delete_child(kind.clone(), *id)
    });

    // ── Commit helpers ──────────────────────────────────────────────────
    let commit_title = Callback::new(move |v: String| {
        title.set(v.clone());
        patch.dispatch(("movie".into(), id, "title".into(), Some(v)));
    });
    let commit_desc = Callback::new(move |v: String| {
        description.set(v.clone());
        let value = if v.is_empty() { None } else { Some(v) };
        patch.dispatch(("movie".into(), id, "description".into(), value));
    });
    let on_poster_file = Callback::new(move |file: web_sys::File| {
        let fd = web_sys::FormData::new().unwrap();
        let _ = fd.append_with_str("kind", "movie");
        let _ = fd.append_with_str("id", &id.to_string());
        let _ = fd.append_with_blob_and_filename("poster_file", &file, &file.name());
        poster_upload.dispatch(fd);
    });
    Effect::new(move |_| {
        if let Some(Ok(url)) = poster_upload.value().get() {
            poster.set(Some(url));
        }
    });

    view! {
        <DetailShell poster=movie.poster.clone()>
            <DetailHero poster=view! {
                <EditablePoster
                    src=Signal::derive(move || poster.get())
                    placeholder=MoviePosterSvg.into()
                    on_file=on_poster_file
                    input_id=format!("poster-movie-{id}")
                />
            }>
                <HeroBadge label="فيلم" icon=MovieIcon()/>
                <EditableText
                    value=Signal::derive(move || title.get())
                    on_commit=commit_title
                    class="text-3xl sm:text-4xl md:text-5xl font-black tracking-tight mb-2 text-white"
                />
                <div class="mt-4 max-w-2xl">
                    <EditableTextArea
                        value=Signal::derive(move || description.get())
                        on_commit=commit_desc
                        placeholder="أضف وصفاً..."
                        class="text-gray-300 leading-relaxed text-base sm:text-lg"
                    />
                </div>
            </DetailHero>

            <div class="mt-10">
                <ChapterPanel
                    movie_id=id
                    chapters=chapters
                    refresh=refresh
                    delete_action=delete_child
                />
            </div>

            {move || {
                let items: Vec<MediaItem> = chapters
                    .get()
                    .and_then(|r| r.ok())
                    .unwrap_or_default()
                    .into_iter()
                    .map(|ch| {
                        let t = ch
                            .title
                            .clone()
                            .unwrap_or_else(|| format!("الفصل {}", ch.number + 1));
                        let mut it = MediaItem::new(ch.id, t, ch.file.path.clone());
                        if let Some(d) = ch.description {
                            it = it.with_subtitle(d);
                        }
                        it
                    })
                    .collect::<Vec<_>>();
                (!items.is_empty()).then(|| view! {
                    <div class="mt-10">
                        <MediaPlayer items=items playlist_title=movie.title.clone()/>
                    </div>
                })
            }}
        </DetailShell>
    }
}

// ─── Chapters panel ─────────────────────────────────────────────────────────

#[component]
fn ChapterPanel(
    movie_id: u64,
    chapters: Resource<Result<Vec<MovieChapter>, ServerFnError>>,
    refresh: RwSignal<u32>,
    delete_action: Action<(String, u64), Result<(), ServerFnError>>,
) -> impl IntoView {
    let upload = Action::new_local(|fd: &web_sys::FormData| {
        crate::app::upload_api::upload_media(fd.clone().into())
    });

    // After a successful upload, refresh the chapters list.
    Effect::new(move |_| {
        if matches!(upload.value().get(), Some(Ok(_))) {
            refresh.update(|n| *n += 1);
        }
    });

    // Stable id — movie_id comes from the route, identical on SSR and hydrate.
    let input_id = format!("chapter-input-{movie_id}");
    let input_id: &'static str = input_id.leak();

    let on_files = move |ev: web_sys::Event| {
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

        let fd = web_sys::FormData::new().unwrap();
        let _ = fd.append_with_str("title", "");
        let _ = fd.append_with_str("description", "");
        let _ = fd.append_with_str("media_type", "movie");
        let _ = fd.append_with_str("is_new", "false");
        let _ = fd.append_with_str("existing_id", &movie_id.to_string());

        for i in 0..files.length() {
            if let Some(f) = files.get(i) {
                let file: web_sys::File = f.unchecked_into();
                let name = file.name();
                let stem = name.rsplitn(2, '.').last().unwrap_or(&name).to_string();
                let _ = fd.append_with_blob_and_filename(&format!("file_{i}"), &file, &name);
                let _ = fd.append_with_str(&format!("file_title_{i}"), &stem);
            }
        }
        upload.dispatch(fd);
        input.set_value("");
    };

    // Unwrapped chapter list; empty until the first fetch resolves.
    let rows = move || chapters.get().and_then(|r| r.ok()).unwrap_or_default();

    view! {
        <div class="space-y-3">
            <div class="flex items-center justify-between">
                <h2 class="text-xl font-bold text-white">"الفصول"</h2>
                <input type="file" id=input_id class="hidden" multiple
                    accept=".mp4,.mkv,.mov,.webm,.avi,.m4v,.wmv,.flv,.ts" on:change=on_files/>
                <label for=input_id class="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-green-500/20 hover:bg-green-500/30 text-green-300 text-sm font-medium cursor-pointer transition">
                    <UploadIcon/> "إضافة فصول"
                </label>
            </div>

            <Show when=move || upload.pending().get()>
                <div class="text-cyan-300 text-sm">"جاري الرفع والتحويل..."</div>
            </Show>

            <Show when=move || delete_action.pending().get()>
                <div class="text-cyan-300 text-sm">"جاري الحذف..."</div>
            </Show>

            <For each=rows key=|c| c.id let:ch>
                <ChapterRow chapter=ch refresh=refresh delete_action=delete_action/>
            </For>
        </div>
    }
}

#[component]
fn ChapterRow(
    chapter: MovieChapter,
    refresh: RwSignal<u32>,
    delete_action: Action<(String, u64), Result<(), ServerFnError>>,
) -> impl IntoView {
    let chapter_id = chapter.id;
    let title = RwSignal::new(chapter.title.clone().unwrap_or_default());
    let number = chapter.number;
    let file_path = chapter.file.path.clone();

    let on_delete = move |_| {
        delete_action.dispatch(("movie_chapter".to_string(), chapter_id));
        // Trigger a refetch after the delete has been dispatched.
        // (The delete itself is fast; a tiny delay avoids racing the DB.)
        refresh.update(|n| *n += 1);
    };

    // Rename chapters by patching `movie_chapters.title` directly.
    // (We don't have a generic child-patch server fn yet, so we add one.)
    let rename = Action::new_local(|(id, title): &(u64, String)| {
        crate::app::media_api::patch_chapter_title(*id, title.clone())
    });
    let on_title_change = move |ev: web_sys::Event| {
        title.set(event_target_value(&ev));
    };
    let on_title_blur = move |_| {
        rename.dispatch((chapter_id, title.get_untracked()));
    };

    view! {
        <div class="bg-white/5 backdrop-blur-sm rounded-xl border border-white/10 p-4 flex items-center gap-3">
            <span class="text-gray-400 text-sm font-mono w-8 shrink-0">
                {number + 1}
            </span>
            <input
                type="text"
                class="flex-1 bg-transparent text-white focus:outline-none focus:bg-white/5 rounded px-2 py-1"
                prop:value=move || title.get()
                on:input=on_title_change
                on:blur=on_title_blur
            />
            <a
                href=file_path
                class="text-cyan-400 hover:text-cyan-300 text-sm shrink-0"
                download
            >
                "تحميل"
            </a>
            <button
                type="button"
                on:click=on_delete
                class="text-red-400 hover:text-red-300 p-1 shrink-0"
                aria-label="حذف"
            >
                <DeleteIcon/>
            </button>
        </div>
    }
}
