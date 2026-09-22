use axum::{
    extract::Request,
    http::{Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use std::sync::OnceLock;

static EXPECTED_TOKEN: OnceLock<Option<String>> = OnceLock::new();

pub fn init_from_env() {
    let token = std::env::var("HOMEX_TOKEN").ok().filter(|t| !t.is_empty());
    if token.is_some() {
        leptos::logging::log!("[auth] HOMEX_TOKEN set — authentication enabled");
    } else {
        leptos::logging::warn!("[auth] HOMEX_TOKEN not set — running WITHOUT authentication");
    }
    let _ = EXPECTED_TOKEN.set(token);
}

/// `None` = no token configured = auth disabled.
pub fn expected_token() -> Option<&'static str> {
    EXPECTED_TOKEN.get().and_then(|o| o.as_deref())
}

fn extract_token(req: &Request) -> Option<String> {
    if let Some(v) = req.headers().get(header::AUTHORIZATION)
        && let Ok(s) = v.to_str()
        && let Some(tok) = s.strip_prefix("Bearer ")
    {
        return Some(tok.to_string());
    }
    if let Some(c) = req.headers().get(header::COOKIE)
        && let Ok(s) = c.to_str()
    {
        for part in s.split(';') {
            if let Some(tok) = part.trim().strip_prefix("homex_token=") {
                return Some(tok.to_string());
            }
        }
    }
    None
}

const PUBLIC_ENDPOINTS: &[&str] = &["/login", "/api/login"];

fn is_public(path: &str) -> bool {
    PUBLIC_ENDPOINTS.contains(&path)
        || path.starts_with("/pkg/")
        || path.starts_with("/posters/")
        || path.starts_with("/media/")
        || matches!(
            path.rsplit('.').next(),
            Some(
                "css"
                    | "js"
                    | "wasm"
                    | "png"
                    | "jpg"
                    | "jpeg"
                    | "webp"
                    | "svg"
                    | "ico"
                    | "woff"
                    | "woff2"
            )
        )
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
        return Redirect::to("/login").into_response();
    }
    (StatusCode::UNAUTHORIZED, "unauthorized").into_response()
}
