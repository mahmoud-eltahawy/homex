use crate::app::types::{PosterUrl, RelPath};
use tokio_util::sync::CancellationToken;

pub mod auth;
pub mod config;
pub mod convert;
pub mod db;
pub mod jobs;
pub mod poster;
pub mod routes;
pub mod upload;

pub use config::Config;
pub use convert::Jobs;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub config: Config,
    pub jobs: Jobs,
    pub cancel: CancellationToken,
}

pub async fn remove_media_files(state: &AppState, rel_paths: &[RelPath]) {
    for rel in rel_paths {
        let abs = rel.join_under(&state.config.storage.media_root);
        if let Err(e) = tokio::fs::remove_file(&abs).await {
            leptos::logging::warn!("[cleanup] remove media {}: {e}", abs.display());
        }
    }
}

pub async fn remove_poster(state: &AppState, poster: Option<&PosterUrl>) {
    let Some(poster) = poster else { return };
    let abs = poster.to_filesystem_path(&state.config.storage.data_dir);
    if let Err(e) = tokio::fs::remove_file(&abs).await {
        leptos::logging::warn!("[cleanup] remove poster {}: {e}", abs.display());
    }
}
