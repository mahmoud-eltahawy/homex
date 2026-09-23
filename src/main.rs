#![recursion_limit = "256"]

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use homex::app::server::auth;
    use leptos::prelude::*;
    use tokio_util::sync::CancellationToken;

    auth::init_from_env();

    let config = load_config_or_exit();
    let db = init_db_or_exit(&config).await;
    ensure_storage_dirs(&config).await;
    log_boot_info(&config);

    let cancel = CancellationToken::new();
    let state = build_app_state(db, config.clone(), cancel.clone());

    let leptos_options = get_configuration(None).unwrap().leptos_options;
    let router = build_router(state, leptos_options);

    run_server(router, config.server.addr, cancel).await;
}

#[cfg(feature = "ssr")]
fn load_config_or_exit() -> homex::app::server::Config {
    use homex::app::server::Config;
    use leptos::logging::error;
    match Config::load() {
        Ok(c) => c,
        Err(e) => {
            error!("[fatal] config: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(feature = "ssr")]
async fn init_db_or_exit(config: &homex::app::server::Config) -> toasty::Db {
    use leptos::logging::error;
    match homex::app::server::db::init(config).await {
        Ok(p) => p,
        Err(e) => {
            error!("[fatal] db init: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(feature = "ssr")]
async fn ensure_storage_dirs(config: &homex::app::server::Config) {
    use homex::app::constants::storage;
    use leptos::logging::error;
    let mut dirs = vec![
        config.storage.media_root.clone(),
        config.storage.data_dir.clone(),
        config.storage.data_dir.join(storage::POSTERS_DIR),
    ];
    for dir in dirs.drain(..) {
        if let Err(e) = tokio::fs::create_dir_all(&dir).await {
            error!("[fatal] mkdir {}: {e}", dir.display());
            std::process::exit(1);
        }
    }
}

#[cfg(feature = "ssr")]
fn log_boot_info(config: &homex::app::server::Config) {
    use leptos::logging::log;
    log!(
        "[boot] media_root={} data_dir={} db={}",
        config.storage.media_root.display(),
        config.storage.data_dir.display(),
        config.db_path().display(),
    );
}

#[cfg(feature = "ssr")]
fn build_app_state(
    db: toasty::Db,
    config: homex::app::server::Config,
    cancel: tokio_util::sync::CancellationToken,
) -> homex::app::server::AppState {
    use homex::app::server::{AppState, convert};
    AppState {
        db,
        config,
        jobs: convert::new_jobs(),
        cancel,
    }
}

#[cfg(feature = "ssr")]
fn build_router(
    state: homex::app::server::AppState,
    leptos_options: leptos::prelude::LeptosOptions,
) -> axum::Router {
    use axum::{Extension, Router, extract::DefaultBodyLimit, middleware, routing::get};
    use homex::app::constants::{routes, storage};
    use homex::app::{App, server::auth, server::routes::stream_media, shell};
    use leptos::prelude::*;
    use leptos_axum::LeptosRoutes;
    use leptos_axum::generate_route_list;
    use tower_http::services::ServeDir;

    let posters_dir = state.config.storage.data_dir.join(storage::POSTERS_DIR);
    let routes_list = generate_route_list(App);

    Router::new()
        .nest_service(routes::POSTERS_SERVE_PREFIX, ServeDir::new(posters_dir))
        .leptos_routes_with_context(
            &leptos_options,
            routes_list,
            {
                let state = state.clone();
                move || provide_context(state.clone())
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .route(routes::MEDIA_STREAM_PATTERN, get(stream_media))
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024 * 1024))
        .layer(Extension(state))
        .layer(middleware::from_fn(auth::middleware))
        .with_state(leptos_options)
}

#[cfg(feature = "ssr")]
async fn run_server(
    router: axum::Router,
    addr: std::net::SocketAddr,
    cancel: tokio_util::sync::CancellationToken,
) {
    use leptos::logging::log;

    let shutdown = shutdown_future(cancel);
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, router.into_make_service())
        .with_graceful_shutdown(shutdown)
        .await
        .unwrap();
}

#[cfg(feature = "ssr")]
async fn shutdown_future(cancel: tokio_util::sync::CancellationToken) {
    use leptos::logging::log;
    shutdown_signal().await;
    log!("[shutdown] cancelling in-flight jobs");
    cancel.cancel();
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
}

#[cfg(feature = "ssr")]
async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let term = async {
        use tokio::signal::unix::{SignalKind, signal};
        match signal(SignalKind::terminate()) {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let term = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {}
        _ = term => {}
    }
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
