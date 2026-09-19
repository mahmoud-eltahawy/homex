use crate::app::{
    detail::{
        DetailHero, DetailShell, DownloadButton, HeroBadge, HeroDescription, HeroMeta, HeroTitle,
        Poster,
    },
    icons::{ClockIcon, MovieIcon, MoviePosterSvg},
    media_player::{MediaItem, MediaPlayer},
    model::{MediaType, Movie, MovieChapter},
    resource_view::ResourceView,
    route_params::use_u64_param,
};
use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};

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
        let movie = Resource::new(use_u64_param("id"), fetch_movie_detail);
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

    let edit_href = MediaType::Movie.edit_href(movie.id);
    view! {
        <DetailShell poster=poster.clone()  edit_href>
            <DetailBody movie=movie.clone() chapter=first_chapter/>
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
    let poster = movie.poster.clone();
    let title = movie.title.clone();
    let description = movie
        .description
        .clone()
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

    let has_download = !download_link.is_empty();
    let download_title = title.clone();

    view! {
        <DetailHero poster=view! {
            <Poster
                src=poster
                alt=title.clone()
                placeholder=view! { <MoviePosterSvg/> }.into_any()
            />
        }>
            <HeroBadge label="فيلم" icon=MovieIcon()/>
            <HeroTitle title=title.clone()/>
            <HeroMeta>
                <span class="flex items-center gap-1"><ClockIcon/>{duration}</span>
                <span>{size}</span>
            </HeroMeta>
            <HeroDescription text=description/>
            {has_download.then(move || view! {
                <div class="mt-6 flex gap-3">
                    <DownloadButton href=download_link download_name=download_title/>
                </div>
            })}
        </DetailHero>
    }
}
