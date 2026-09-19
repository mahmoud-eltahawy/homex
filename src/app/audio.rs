use crate::app::{
    detail::Poster,
    icons::{AudioIcon, MusicPosterSvg},
    model::{Audio, AudioGroup, MediaType},
    view_schema::CardData,
};
use leptos::prelude::*;

pub mod detail;
pub mod song;

#[component]
pub fn AudioArtwork(poster: Option<String>, title: String) -> impl IntoView {
    let placeholder = view! {
        <div class="w-full aspect-square rounded-2xl border border-white/10 bg-gradient-to-br from-cyan-500/20 via-purple-500/20 to-pink-500/20 flex items-center justify-center overflow-hidden">
            <div class="text-cyan-300/80 origin-center scale-[6]">
                <AudioIcon/>
            </div>
        </div>
    }
    .into_any();

    view! {
        <Poster
            src=poster
            alt=title
            class="w-full aspect-square object-cover rounded-2xl shadow-2xl border border-white/10".to_string()
            placeholder=placeholder
        />
    }
}

impl CardData for AudioGroup {
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
        MediaType::AudioGroup
    }

    fn badge_icon() -> impl IntoView {
        AudioIcon()
    }

    fn placeholder_poster() -> impl IntoView {
        MusicPosterSvg()
    }
}

#[server]
pub async fn fetch_audio_groups(
    offset: usize,
    size: usize,
    search_query: Option<String>,
) -> Result<Vec<AudioGroup>, ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();

    #[derive(sqlx::FromRow)]
    struct Row {
        id: i64,
        title: String,
        poster: Option<String>,
        description: Option<String>,
        audios_count: i64,
    }
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT g.id, g.title, g.poster, g.description, \
                (SELECT COUNT(*) FROM audios a WHERE a.group_id = g.id) AS audios_count \
         FROM audio_groups g \
         WHERE (?1 IS NULL OR g.title LIKE '%' || ?1 || '%') \
         ORDER BY g.title LIMIT ?2 OFFSET ?3",
    )
    .bind(&search_query)
    .bind(size as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|r| AudioGroup {
            id: r.id as u64,
            title: r.title,
            poster: r.poster,
            description: r.description,
            audios_count: r.audios_count as u32,
        })
        .collect())
}

#[server]
pub async fn fetch_audio_groups_count(
    search_query: Option<String>,
) -> Result<usize, ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audio_groups \
         WHERE (?1 IS NULL OR title LIKE '%' || ?1 || '%')",
    )
    .bind(&search_query)
    .fetch_one(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(n as usize)
}

#[server]
async fn fetch_audio_group_detail(id: u64) -> Result<AudioGroup, ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();

    #[derive(sqlx::FromRow)]
    struct Row {
        id: i64,
        title: String,
        poster: Option<String>,
        description: Option<String>,
        audios_count: i64,
    }
    let r: Row = sqlx::query_as(
        "SELECT g.id, g.title, g.poster, g.description, \
                (SELECT COUNT(*) FROM audios a WHERE a.group_id = g.id) AS audios_count \
         FROM audio_groups g WHERE g.id = ?",
    )
    .bind(id as i64)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("audio group not found"))?;

    Ok(AudioGroup {
        id: r.id as u64,
        title: r.title,
        poster: r.poster,
        description: r.description,
        audios_count: r.audios_count as u32,
    })
}

#[server]
pub async fn fetch_audios(group_id: u64) -> Result<Vec<Audio>, ServerFnError> {
    use crate::app::model::MediaFile;
    use crate::app::server::AppState;
    let state: AppState = expect_context();

    #[derive(sqlx::FromRow)]
    struct Row {
        id: i64,
        title: String,
        file_id: i64,
        size: i64,
        dur: i64,
    }
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT a.id, a.title, f.id AS file_id, f.size_bytes AS size, f.duration_secs AS dur \
         FROM audios a JOIN files f ON f.id = a.file_id \
         WHERE a.group_id = ? ORDER BY a.number",
    )
    .bind(group_id as i64)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|r| Audio {
            id: r.id as u64,
            title: r.title,
            file: MediaFile {
                id: r.file_id as u64,
                path: format!("/media/{}", r.file_id),
                size: r.size as u64,
                duration: r.dur as u64,
            },
        })
        .collect())
}

#[server]
pub async fn fetch_audio(group_id: u64, song_id: u64) -> Result<Audio, ServerFnError> {
    use crate::app::model::MediaFile;
    use crate::app::server::AppState;
    let state: AppState = expect_context();

    #[derive(sqlx::FromRow)]
    struct Row {
        id: i64,
        title: String,
        file_id: i64,
        size: i64,
        dur: i64,
    }
    let r: Row = sqlx::query_as(
        "SELECT a.id, a.title, f.id AS file_id, f.size_bytes AS size, f.duration_secs AS dur \
         FROM audios a JOIN files f ON f.id = a.file_id \
         WHERE a.group_id = ? AND a.id = ?",
    )
    .bind(group_id as i64)
    .bind(song_id as i64)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("audio not found"))?;

    Ok(Audio {
        id: r.id as u64,
        title: r.title,
        file: MediaFile {
            id: r.file_id as u64,
            path: format!("/media/{}", r.file_id),
            size: r.size as u64,
            duration: r.dur as u64,
        },
    })
}
