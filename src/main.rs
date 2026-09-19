#![recursion_limit = "256"]

use homex::app::server::{AppState, Config};

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::{Extension, Router, routing::get};
    use homex::app::{server::routes::stream_media, *};
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{LeptosRoutes, generate_route_list};
    use tower_http::services::ServeDir;

    let config = Config::load().expect("failed to load homex.toml");
    let server_addr = config.server.addr;

    let db = server::db::init(&config)
        .await
        .expect("failed to init database");

    let state = AppState { db, config };

    let posters_dir = state.config.storage.data_dir.join("posters");
    tokio::fs::create_dir_all(&posters_dir)
        .await
        .expect("failed to create posters dir");

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
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
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &server_addr);
    let listener = tokio::net::TcpListener::bind(&server_addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
