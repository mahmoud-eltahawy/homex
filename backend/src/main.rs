use std::{
    env::{self, home_dir},
    net::SocketAddr,
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

use axum::{
    Router,
    extract::Request,
    http::{
        HeaderValue, Method, StatusCode,
        header::{ACCEPT, CONTENT_TYPE},
    },
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use tokio::net::TcpListener;
use tower_http::{
    CompressionLevel,
    compression::{CompressionLayer, predicate::SizeAbove},
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{Level, info};
use tracing_subscriber::{filter, layer::SubscriberExt, util::SubscriberInitExt};
use turso::Database;

struct AppState {
    db: Database,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // SECTION : tracing
    let filter = filter::Targets::new()
        .with_default(Level::INFO)
        .with_target("tower_http", Level::WARN)
        .with_target("server", Level::DEBUG)
        .with_target("turso", Level::DEBUG);

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .init();
    //SECTION : Database
    let db = turso::Builder::new_local("homex_data.db").build().await?;

    //SECTION : Database
    let state = Arc::new(AppState { db });

    //SECTION : listener
    let addr = {
        let host = env::var("HOMEX_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("HOMEX_PORT").unwrap_or_else(|_| "3000".to_string());

        format!("{host}:{port}").parse::<SocketAddr>()?
    };
    let listener = TcpListener::bind(&addr).await?;
    info!("listening at {}", addr);

    let router = Router::new()
        .nest("/api", api_handler(Arc::clone(&state)))
        .merge(static_file_handler());

    axum::serve(
        listener,
        router
            .layer(cors_layer()?)
            .layer(
                CompressionLayer::new()
                    .quality(CompressionLevel::Precise(4))
                    .compress_when(SizeAbove::new(512)),
            )
            .layer(TraceLayer::new_for_http())
            .into_make_service(),
    )
    .await?;
    Ok(())
}

fn cors_layer() -> Result<CorsLayer, Box<dyn std::error::Error>> {
    let res = CorsLayer::new()
        .allow_headers([ACCEPT, CONTENT_TYPE])
        .max_age(Duration::from_secs(86400))
        .allow_origin(
            std::env::var("CORS_ORIGIN")
                .unwrap_or_else(|_| "*".to_string())
                .parse::<HeaderValue>()?,
        )
        .allow_methods(vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
            Method::HEAD,
            Method::PATCH,
        ]);
    Ok(res)
}

fn api_handler(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(|| async { (StatusCode::OK, "OK") }))
        .with_state(state)
}

fn static_file_handler() -> Router {
    let Ok(public_dir) = env::var("PUBLIC_DIR")
        .unwrap_or("Downloads".to_string())
        .parse::<PathBuf>();
    let mut home = home_dir().expect("home directory can not be found!!!");
    home.push(public_dir);
    let public_dir = home.canonicalize().expect("public dir path does not exist");
    let public_dir = ServeDir::new(public_dir);
    #[cfg(not(debug_assertions))]
    let site_service = ServeDir::new("./").not_found_service(ServeFile::new("./index.html"));
    #[cfg(debug_assertions)]
    let site_service = ServeDir::new("../homex/dist")
        .not_found_service(ServeFile::new("../homex/dist/index.html"));
    Router::new()
        .nest_service("/homex", public_dir)
        .fallback_service(site_service)
        .layer(middleware::from_fn(cache_control))
}

async fn cache_control(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;

    if let Some(content_type) = response.headers().get(CONTENT_TYPE) {
        const CACHEABLE_CONTENT_TYPES: [&str; 6] = [
            "text/css",
            "text/javascript",
            "image/svg+xml",
            "image/webp",
            "font/woff2",
            "image/png",
        ];

        if CACHEABLE_CONTENT_TYPES.iter().any(|&ct| content_type == ct) {
            let value = HeaderValue::from_str("public, max-age=86400").unwrap();
            response.headers_mut().insert("cache-control", value);
        }
    }

    response
}
