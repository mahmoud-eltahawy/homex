use crate::app::constants::{STATIC_ASSET_EXTS, auth as auth_consts, routes};
use axum::{
    extract::Request,
    http::{Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use std::sync::OnceLock;

static EXPECTED_TOKEN: OnceLock<Option<String>> = OnceLock::new();

pub fn init_from_env() {
    let token = std::env::var(auth_consts::ENV_TOKEN)
        .ok()
        .filter(|t| !t.is_empty());
    if token.is_some() {
        leptos::logging::log!("[auth] HOMEX_TOKEN set — authentication enabled");
    } else {
        leptos::logging::warn!("[auth] HOMEX_TOKEN not set — running WITHOUT authentication");
    }
    let _ = EXPECTED_TOKEN.set(token);
}

pub fn expected_token() -> Option<&'static str> {
    EXPECTED_TOKEN.get().and_then(|o| o.as_deref())
}

fn extract_token(req: &Request) -> Option<String> {
    if let Some(v) = req.headers().get(header::AUTHORIZATION)
        && let Ok(s) = v.to_str()
        && let Some(tok) = s.strip_prefix(auth_consts::BEARER_PREFIX)
    {
        return Some(tok.to_string());
    }
    if let Some(c) = req.headers().get(header::COOKIE)
        && let Ok(s) = c.to_str()
    {
        for part in s.split(';') {
            if let Some(tok) = part.trim().strip_prefix(auth_consts::COOKIE_PREFIX) {
                return Some(tok.to_string());
            }
        }
    }
    None
}

const PUBLIC_ENDPOINTS: &[&str] = &[routes::LOGIN, routes::LOGIN_API];

fn is_public(path: &str) -> bool {
    PUBLIC_ENDPOINTS.contains(&path)
        || path.starts_with(routes::PKG_URL_PREFIX)
        || path.starts_with(routes::POSTERS_URL_PREFIX)
        || path.starts_with(routes::MEDIA_URL_PREFIX)
        || path
            .rsplit('.')
            .next()
            .is_some_and(|ext| STATIC_ASSET_EXTS.contains(&ext))
}

pub async fn middleware(req: Request, next: Next) -> Response {
    let Some(expected) = expected_token() else {
        return next.run(req).await;
    };

    let method = req.method().clone();
    let path = req.uri().path().to_string();

    if is_public(&path) {
        return next.run(req).await;
    }
    if extract_token(&req).as_deref() == Some(expected) {
        return next.run(req).await;
    }

    let wants_html = req
        .headers()
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|s| s.contains("text/html"));

    if wants_html && method == Method::GET {
        return Redirect::to(routes::LOGIN).into_response();
    }
    (StatusCode::UNAUTHORIZED, "unauthorized").into_response()
}
