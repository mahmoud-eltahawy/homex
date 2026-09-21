use axum::{
    Extension,
    body::Body,
    extract::Path,
    http::{Request, StatusCode},
    response::{IntoResponse, Redirect, Response},
};
use tower::ServiceExt;
use tower_http::services::ServeFile;

use crate::app::server::db;

use super::AppState;

pub async fn stream_media(
    Extension(state): Extension<AppState>,
    Path(file_id): Path<i64>,
    req: Request<Body>,
) -> Response {
    let rel: Option<String> = match db::fetch_file_path(&state.db, file_id).await {
        Ok(r) => r,
        Err(e) => {
            leptos::logging::error!("[stream] db error: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let Some(rel) = rel else {
        return StatusCode::NOT_FOUND.into_response();
    };

    // Absolute URLs (demo seed data) → redirect to origin.
    if rel.starts_with("http://") || rel.starts_with("https://") {
        return Redirect::temporary(&rel).into_response();
    }

    let abs = state.config.storage.media_root.join(&rel);
    if tokio::fs::metadata(&abs).await.is_err() {
        leptos::logging::warn!("[stream] missing file: {}", abs.display());
        return StatusCode::NOT_FOUND.into_response();
    }

    match ServeFile::new(&abs).oneshot(req).await {
        Ok(res) => res.into_response(),
        Err(e) => {
            leptos::logging::error!("[stream] serve error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
