use leptos::prelude::ServerFnError;
use sqlx::SqlitePool;
use tokio_util::sync::CancellationToken;

pub mod auth;
pub mod config;
pub mod convert;
pub mod db;
pub mod poster;
pub mod routes;
pub mod upload;

pub use config::Config;
pub use convert::Jobs;

#[derive(Clone)]
pub struct AppState {
    pub db: SqlitePool,
    pub config: Config,
    pub jobs: Jobs,
    pub cancel: CancellationToken,
}

pub trait SqlErr<T> {
    fn srv(self) -> Result<T, ServerFnError>;
}

impl<T> SqlErr<T> for Result<T, sqlx::Error> {
    fn srv(self) -> Result<T, ServerFnError> {
        self.map_err(|e| {
            leptos::logging::error!("[db] {e}");
            ServerFnError::new("حدث خطأ داخلي")
        })
    }
}

pub async fn remove_media_files(state: &AppState, rel_paths: &[String]) {
    for rel in rel_paths {
        if rel.starts_with("http://") || rel.starts_with("https://") {
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
    let Some(rest) = url.strip_prefix("/posters/") else {
        return;
    };
    let abs = state.config.storage.data_dir.join("posters").join(rest);
    if let Err(e) = tokio::fs::remove_file(&abs).await {
        leptos::logging::warn!("[cleanup] remove poster {}: {e}", abs.display());
    }
}
