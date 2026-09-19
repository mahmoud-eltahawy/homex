use crate::app::{
    detail::DetailShell,
    icons::{ClockIcon, DownloadIcon, MovieIcon},
    media_player::{MediaItem, MediaPlayer},
    model::{Movie, MovieChapter},
    resource_view::ResourceView,
    view_schema::CardData,
};
use leptos::prelude::*;
use leptos_router::{LazyRoute, hooks::use_params_map, lazy_route};

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

pub struct MovieDetailPage {
    movie: Resource<Result<Movie, ServerFnError>>,
}

#[lazy_route]
impl LazyRoute for MovieDetailPage {
    fn data() -> Self {
        let params = use_params_map();
        let id =
            move || params.with(|p| p.get("id").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0));
        let movie = Resource::new(id, fetch_movie_detail);
        Self { movie }
    }

    fn view(this: Self) -> AnyView {
        let adapter = |movie| MovieDetailProps { movie };
        view! {
            <ResourceView resource=this.movie view_fn=MovieDetail adapter=adapter />
        }
        .into_any()
    }
}

#[component]
fn MovieDetail(movie: Movie) -> impl IntoView {
    let items: Vec<MediaItem> = movie
        .chapters
        .iter()
        .map(|ch| {
            let title = ch
                .title
                .clone()
                .unwrap_or_else(|| format!("الفصل {}", ch.number + 1));
            let mut item = MediaItem::new(ch.id, title, ch.file.path.clone());
            if let Some(d) = ch.description.clone() {
                item = item.with_subtitle(d);
            }
            item
        })
        .collect();

    let has_items = !items.is_empty();
    let first_chapter = movie.chapters.first().cloned();
    let poster = movie.poster.clone();

    view! {
        <DetailShell poster=poster.clone()>
            <DetailBody movie=movie.clone() chapter=first_chapter />
            {has_items.then(move || {
                view! {
                    <div class="mt-10">
                        <MediaPlayer
                            items=items
                            playlist_title="فصول الفيلم".to_string()
                        />
                    </div>
                }
            })}
        </DetailShell>
    }
}

#[component]
fn DetailBody(movie: Movie, chapter: Option<MovieChapter>) -> impl IntoView {
    view! {
        <div class="flex flex-col lg:flex-row gap-8 lg:gap-12 items-start">
            <div class="flex-shrink-0 w-40 sm:w-48 md:w-56 lg:w-64 mx-auto lg:mx-0">
                {movie.clone().poster()}
            </div>
            <div class="flex-1 w-full">
                <DetailMetaBadge/>
                <DetailInfo movie=movie.clone() chapter=chapter/>
            </div>
        </div>
    }
}

#[component]
fn DetailMetaBadge() -> impl IntoView {
    view! {
        <div class="inline-flex items-center gap-2 bg-white/10 backdrop-blur-md rounded-full px-3 py-1 text-sm font-medium mb-4 border border-white/5">
            <MovieIcon/>
            "فيلم"
        </div>
    }
}

#[component]
fn DetailInfo(movie: Movie, chapter: Option<MovieChapter>) -> impl IntoView {
    let title = movie.title.to_string();
    let description = movie
        .description
        .unwrap_or_else(|| "لا يوجد وصف متاح.".to_string());

    let download_link = chapter
        .as_ref()
        .map(|ch| ch.file.path.clone())
        .unwrap_or_default();
    let duration = chapter
        .as_ref()
        .map(|ch| ch.file.human_readable_duration())
        .unwrap_or_default();
    let size = chapter
        .as_ref()
        .map(|ch| ch.file.human_readable_size())
        .unwrap_or_default();

    view! {
        <h1 class="text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-black tracking-tight mb-2">
            {title}
        </h1>
        <div class="flex flex-wrap items-center gap-3 sm:gap-4 text-gray-300 mt-2 mb-6 text-sm sm:text-base">
            <span class="flex items-center gap-1"><ClockIcon/>{duration}</span>
            <span>{size}</span>
        </div>
        <p class="text-gray-300 leading-relaxed max-w-2xl text-base sm:text-lg">{description}</p>
        <div class="mt-6 flex gap-3">
            <a
                download="download"
                href=download_link
                class="inline-flex items-center gap-2 bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold py-2.5 px-6 rounded-2xl shadow-lg shadow-cyan-500/20 transition-all hover:scale-105 hover:shadow-cyan-500/40 text-sm"
            >
                <DownloadIcon/> "تحميل"
            </a>
        </div>
    }
}
