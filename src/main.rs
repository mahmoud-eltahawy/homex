#![recursion_limit = "256"]

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::middleware;
    use axum::{Extension, Router, routing::get};
    use homex::app::server::auth;
    use homex::app::server::{AppState, Config};
    use homex::app::{server::routes::stream_media, *};
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
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

    let state = AppState {
        db,
        config,
        jobs: server::convert::new_jobs(),
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
        .layer(Extension(state))
        .layer(middleware::from_fn(auth::middleware))
        .with_state(leptos_options);

    log!("listening on http://{}", &server_addr);
    let listener = tokio::net::TcpListener::bind(&server_addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
