use crate::app::{
    collection_detail::CollectionDetailPage, home::HomePage, layout::Layout,
    section_listing::SectionListingPage,
};
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    Lazy,
    components::{ParentRoute, Route, Router, Routes},
    path,
};

mod collection_detail;
mod collections;
mod home;
mod layout;
mod section_listing;
mod sections;

mod common;
pub mod detail;
pub mod icons;
mod inline_edit;
mod media_player;
mod model;
mod pagination;
mod resource_view;
mod route_params;
mod search;
#[cfg(feature = "ssr")]
pub mod server;
mod upload_api;
mod upload_job;

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
        <Stylesheet id="leptos" href="/pkg/homex.css"/>
        <Title text="HomeX"/>
        <Router>
            <Routes fallback=|| "Page not found.".into_view()>
                <ParentRoute path=path!("") view=Layout>
                    <Route path=path!("/")            view={Lazy::<HomePage>::new()}/>
                    <Route path=path!("/s/:slug")     view={Lazy::<SectionListingPage>::new()}/>
                    <Route path=path!("/s/:slug/:id") view={Lazy::<CollectionDetailPage>::new()}/>
                    <Route
                        path=path!("/s/:slug/:id/item/:item_id")
                        view={Lazy::<CollectionDetailPage>::new()}
                    />
                </ParentRoute>
            </Routes>
        </Router>
    }
}
