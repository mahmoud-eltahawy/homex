use super::model::MediaType;
use crate::app::{
    icons::{MovieIcon, MoviePosterSvg},
    model::Movie,
    view_schema::CardData,
};
use leptos::prelude::*;

pub mod detail;

impl CardData for Movie {
    fn id(&self) -> u64 {
        self.id
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn poster_url(&self) -> Option<&str> {
        self.poster.as_deref()
    }

    fn media_type() -> MediaType {
        MediaType::Movie
    }

    fn badge_icon() -> impl IntoView {
        MovieIcon()
    }

    fn placeholder_poster() -> impl IntoView {
        MoviePosterSvg()
    }
}

#[server]
pub async fn fetch_movies(
    offset: usize,
    size: usize,
    search_query: Option<String>,
) -> Result<Vec<Movie>, ServerFnError> {
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

    let rows: Vec<Row> = sqlx::query_as(
        "SELECT id, title, poster, description FROM movies \
         WHERE (?1 IS NULL OR title LIKE '%' || ?1 || '%') \
         ORDER BY title LIMIT ?2 OFFSET ?3",
    )
    .bind(&search_query)
    .bind(size as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut movies = Vec::with_capacity(rows.len());
    for r in rows {
        #[derive(sqlx::FromRow)]
        struct ChRow {
            id: i64,
            number: i64,
            title: Option<String>,
            poster: Option<String>,
            description: Option<String>,
            file_id: i64,
            //path: String,
            size: i64,
            dur: i64,
        }
        let chapters: Vec<ChRow> = sqlx::query_as(
            "SELECT mc.id, mc.number, mc.title, mc.poster, mc.description, \
                    f.id AS file_id, f.relative_path AS path, f.size_bytes AS size, \
                    f.duration_secs AS dur \
             FROM movie_chapters mc JOIN files f ON f.id = mc.file_id \
             WHERE mc.movie_id = ? ORDER BY mc.number",
        )
        .bind(r.id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        movies.push(Movie {
            id: r.id as u64,
            title: r.title,
            poster: r.poster,
            description: r.description,
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
        });
    }
    Ok(movies)
}

#[server]
pub async fn fetch_movies_count(search_query: Option<String>) -> Result<usize, ServerFnError> {
    use crate::app::server::AppState;

    let state: AppState = expect_context();

    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM movies WHERE (?1 IS NULL OR title LIKE '%' || ?1 || '%')",
    )
    .bind(&search_query)
    .fetch_one(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(n as usize)
}
