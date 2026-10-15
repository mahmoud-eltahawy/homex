use crate::app::constants::{self, protocols, storage};
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

pub use crate::app::error::{AppError, ErrorContext};

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub config: Config,
    pub jobs: Jobs,
    pub cancel: CancellationToken,
}

pub async fn remove_media_files(state: &AppState, rel_paths: &[String]) {
    for rel in rel_paths {
        if rel.starts_with(protocols::HTTP) || rel.starts_with(protocols::HTTPS) {
            continue;
        }
        let abs = state.config.storage.media_root.join(rel);
        if let Err(e) = tokio::fs::remove_file(&abs).await {
            leptos::logging::warn!("[cleanup] remove media {}: {e}", abs.display());
        }
    }
}

pub async fn remove_poster(state: &AppState, url: Option<&str>) {
    let Some(url) = url else { return };
    let Some(rest) = url.strip_prefix(constants::routes::POSTERS_URL_PREFIX) else {
        return;
    };
    let abs = state
        .config
        .storage
        .data_dir
        .join(storage::POSTERS_DIR)
        .join(rest);
    if let Err(e) = tokio::fs::remove_file(&abs).await {
        leptos::logging::warn!("[cleanup] remove poster {}: {e}", abs.display());
    }
}
