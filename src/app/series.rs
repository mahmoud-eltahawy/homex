use crate::app::{
    icons::{SeriesIcon, SeriesPosterSvg},
    model::{MediaType, Season, Series},
    view_schema::CardData,
};
use leptos::prelude::*;

pub mod details;

impl CardData for Series {
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
        MediaType::Series
    }

    fn badge_icon() -> impl IntoView {
        SeriesIcon()
    }

    fn placeholder_poster() -> impl IntoView {
        SeriesPosterSvg()
    }
}

#[server]
pub async fn fetch_series(
    offset: usize,
    size: usize,
    search_query: Option<String>,
) -> Result<Vec<Series>, ServerFnError> {
    use crate::app::model::{SeasonSummary, Series};
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
        "SELECT id, title, poster, description FROM series \
         WHERE (?1 IS NULL OR title LIKE '%' || ?1 || '%') \
         ORDER BY title LIMIT ?2 OFFSET ?3",
    )
    .bind(&search_query)
    .bind(size as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let mut out = Vec::with_capacity(rows.len());
    for r in rows {
        #[derive(sqlx::FromRow)]
        struct SeasonRow {
            number: i64,
            episodes: i64,
        }
        let seasons: Vec<SeasonRow> = sqlx::query_as(
            "SELECT s.number, COUNT(e.id) AS episodes \
             FROM seasons s LEFT JOIN episodes e ON e.season_id = s.id \
             WHERE s.series_id = ? GROUP BY s.id ORDER BY s.number",
        )
        .bind(r.id)
        .fetch_all(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        out.push(Series {
            id: r.id as u64,
            title: r.title,
            poster: r.poster,
            description: r.description,
            season_count: seasons.len() as u32,
            season_summaries: seasons
                .into_iter()
                .map(|s| SeasonSummary {
                    season_number: s.number as u32,
                    episode_count: s.episodes as u32,
                })
                .collect(),
        });
    }
    Ok(out)
}

#[server]
pub async fn fetch_series_count(search_query: Option<String>) -> Result<usize, ServerFnError> {
    use crate::app::server::AppState;

    let state: AppState = expect_context();

    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM series WHERE (?1 IS NULL OR title LIKE '%' || ?1 || '%')",
    )
    .bind(&search_query)
    .fetch_one(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(n as usize)
}

#[server]
async fn fetch_series_detail(id: u64) -> Result<Series, ServerFnError> {
    use crate::app::model::{SeasonSummary, Series};
    use crate::app::server::AppState;

    let state: AppState = expect_context();

    #[derive(sqlx::FromRow)]
    struct Row {
        id: i64,
        title: String,
        poster: Option<String>,
        description: Option<String>,
    }
    let r: Row = sqlx::query_as("SELECT id, title, poster, description FROM series WHERE id = ?")
        .bind(id as i64)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("not found"))?;

    #[derive(sqlx::FromRow)]
    struct SeasonRow {
        number: i64,
        episodes: i64,
    }
    let seasons: Vec<SeasonRow> = sqlx::query_as(
        "SELECT s.number, COUNT(e.id) AS episodes \
         FROM seasons s LEFT JOIN episodes e ON e.season_id = s.id \
         WHERE s.series_id = ? GROUP BY s.id ORDER BY s.number",
    )
    .bind(r.id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(Series {
        id: r.id as u64,
        title: r.title,
        poster: r.poster,
        description: r.description,
        season_count: seasons.len() as u32,
        season_summaries: seasons
            .into_iter()
            .map(|s| SeasonSummary {
                season_number: s.number as u32,
                episode_count: s.episodes as u32,
            })
            .collect(),
    })
}

#[server]
pub async fn fetch_season(series_id: u64, season_number: u32) -> Result<Season, ServerFnError> {
    use crate::app::model::{Episode, MediaFile, Season};
    use crate::app::server::AppState;

    let state: AppState = expect_context();

    let season_id: i64 =
        sqlx::query_scalar("SELECT id FROM seasons WHERE series_id = ? AND number = ?")
            .bind(series_id as i64)
            .bind(season_number as i64)
            .fetch_optional(&state.db)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
            .ok_or_else(|| ServerFnError::new("season not found"))?;

    #[derive(sqlx::FromRow)]
    struct EpRow {
        id: i64,
        number: i64,
        file_id: i64,
        size: i64,
        dur: i64,
    }
    let eps: Vec<EpRow> = sqlx::query_as(
        "SELECT e.id, e.number, e.title, f.id AS file_id, \
                f.size_bytes AS size, f.duration_secs AS dur \
         FROM episodes e JOIN files f ON f.id = e.file_id \
         WHERE e.season_id = ? ORDER BY e.number",
    )
    .bind(season_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(Season {
        season_number,
        episodes: eps
            .into_iter()
            .map(|e| Episode {
                id: e.id,
                season: season_number,
                episode: e.number as u32,
                file: MediaFile {
                    id: e.file_id as u64,
                    path: format!("/media/{}", e.file_id),
                    size: e.size as u64,
                    duration: e.dur as u64,
                },
            })
            .collect(),
    })
}
