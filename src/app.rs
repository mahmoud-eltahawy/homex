use crate::app::{
    audio::{detail::AudioGroupDetailPage, song::AudioSongDetailPage},
    home::HomePage,
    layout::Layout,
    listing::{AudioGroupListingPage, MovieListingPage, SeriesListingPage},
    movies::detail::MovieDetailPage,
    series::details::SeriesDetailPage,
    settings::SettingsPage,
};
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    Lazy,
    components::{ParentRoute, Route, Router, Routes},
    path,
};

mod audio;
mod common;
pub mod detail;
mod home;
pub mod icons;
mod inline_edit;
mod layout;
mod listing;
mod media_api;
mod media_player;
mod model;
mod movies;
mod pagination;
mod resource_view;
mod route_params;
mod search;
mod series;
#[cfg(feature = "ssr")]
pub mod server;
mod settings;
mod upload_api;
mod upload_job;
mod view_schema;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="ar" dir="rtl">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Stylesheet id="leptos" href="/pkg/mydisc.css"/>
        <Title text="وسائطي - سينماك الشخصية"/>
        <Router>
            <Routes fallback=|| "Page not found.".into_view()>
                <ParentRoute path=path!("") view=Layout>
                    <Route
                        path=path!("/")
                        view={Lazy::<HomePage>::new()}
                    />
                    <Route
                        path=path!("/series/detail/:id")
                        view={Lazy::<SeriesDetailPage>::new()}
                    />
                    <Route
                        path=path!("/movie/detail/:id")
                        view={Lazy::<MovieDetailPage>::new()}
                    />
                    <Route
                        path=path!("/movie")
                        view={Lazy::<MovieListingPage>::new()}
                    />
                    <Route
                        path=path!("/series")
                        view={Lazy::<SeriesListingPage>::new()}
                    />
                    <Route
                        path=path!("/audio")
                        view={Lazy::<AudioGroupListingPage>::new()}
                    />
                    <Route
                        path=path!("/settings")
                        view={Lazy::<SettingsPage>::new()}
                    />
                    <Route
                        path=path!("/audio/detail/:id")
                        view={Lazy::<AudioGroupDetailPage>::new()}
                    />
                    <Route
                        path=path!("/audio/detail/:id/song/:song_id")
                        view={Lazy::<AudioSongDetailPage>::new()}
                    />
                </ParentRoute>
            </Routes>
        </Router>
    }
}
