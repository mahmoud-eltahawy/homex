#![recursion_limit = "256"]

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{Extension, Router, extract::DefaultBodyLimit, middleware, routing::get};
    use homex::app::server::auth;
    use homex::app::server::{AppState, Config};
    use homex::app::{server::routes::stream_media, *};
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use tokio_util::sync::CancellationToken;
    use tower_http::services::ServeDir;

    auth::init_from_env();

    let config = Config::load().expect("failed to load homex.toml");
    let server_addr = config.server.addr;

    let db = server::db::init(&config)
        .await
        .expect("failed to init database");

    let posters_dir = config.storage.data_dir.join("posters");
    tokio::fs::create_dir_all(&posters_dir)
        .await
        .expect("failed to create posters dir");

    let cancel = CancellationToken::new();

    let state = AppState {
        db,
        config,
        jobs: server::convert::new_jobs(),
        cancel: cancel.clone(),
    };

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let routes = generate_route_list(App);

    let app = Router::new()
        .nest_service("/posters", ServeDir::new(posters_dir))
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            {
                let state = state.clone();
                move || {
                    provide_context(state.clone());
                }
            },
            {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            },
        )
        .route("/media/{id}", get(stream_media))
        .fallback(leptos_axum::file_and_error_handler(shell))
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024 * 1024))
        .layer(Extension(state))
        .layer(middleware::from_fn(auth::middleware))
        .with_state(leptos_options);

    let shutdown = {
        let cancel = cancel.clone();
        async move {
            shutdown_signal().await;
            log!("[shutdown] cancelling in-flight jobs");
            cancel.cancel();
            // Give background tasks a moment to kill their ffmpeg children.
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    };

    log!("listening on http://{}", &server_addr);
    let listener = tokio::net::TcpListener::bind(&server_addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(shutdown)
        .await
        .unwrap();
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
