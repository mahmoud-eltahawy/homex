use axum::{
    Extension,
    body::Body,
    extract::Path,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use tower::ServiceExt;
use tower_http::services::ServeFile;
use uuid::Uuid;

use crate::app::server::db;
use crate::app::types::RelPath;

use super::AppState;

pub async fn stream_media(
    Extension(state): Extension<AppState>,
    Path(file_id): Path<Uuid>,
    req: Request<Body>,
) -> Response {
    let rel: Option<RelPath> = match db::fetch_file_path(&state.db, file_id).await {
        Ok(r) => r,
        Err(e) => {
            leptos::logging::error!("[stream] db error: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let Some(rel) = rel else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let abs = rel.join_under(&state.config.storage.media_root);
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
