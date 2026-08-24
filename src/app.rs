use crate::app::{
    home::HomePage,
    layout::Layout,
    listing::{AudioGroupListingPage, MovieListingPage, SeriesListingPage},
    movies::detail::MovieDetailPage,
    series::details::SeriesDetailPage,
    settings::SettingsPage,
    upload::UploadPage,
};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    path, Lazy,
};

mod view_schema;

mod audio;
mod common;
mod home;
mod icons;
mod layout;
mod listing;
mod model;
mod movies;
mod resource_view;
mod series;
mod settings;
mod upload;
mod video_player;
//TODO : DELETE this
#[cfg(feature = "ssr")]
mod mockary;

//TODO : DELETE this
#[cfg(feature = "ssr")]
async fn delay(ms: i32) {
    tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
}

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
                    <Route path=path!("/") view={Lazy::<HomePage>::new()}/>
                    <Route path=path!("/movie") view={Lazy::<MovieListingPage>::new()}/>
                    <Route path=path!("/series") view={Lazy::<SeriesListingPage>::new()}/>
                    <Route path=path!("/audio") view={Lazy::<AudioGroupListingPage>::new()}/>
                    <Route path=path!("/upload") view={Lazy::<UploadPage>::new()}/>
                    <Route path=path!("/settings") view={Lazy::<SettingsPage>::new()}/>
                    <Route path=path!("/detail/series/:id") view={Lazy::<SeriesDetailPage>::new()}/>
                    <Route path=path!("/detail/movie/:id") view={Lazy::<MovieDetailPage>::new()}/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
