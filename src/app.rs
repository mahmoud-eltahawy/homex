use crate::app::constants::{APP_TITLE, HTML_DIR, HTML_LANG, STYLESHEET_URL};
use crate::app::{
    collection_detail::CollectionDetailPage, home::HomePage, layout::Layout, login::LoginPage,
    section_listing::SectionListingPage,
};
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    Lazy,
    components::{ParentRoute, Route, Router, Routes},
};

mod collection_detail;
mod collections;
mod home;
mod layout;
mod login;
mod section_listing;
mod sections;

mod common;
pub mod constants;
pub mod detail;
mod icons;
mod inline_edit;
mod model;
mod pagination;
mod resource_view;
mod route_params;
mod search;
#[cfg(feature = "ssr")]
pub mod server;
mod upload_api;
mod upload_job;

// ─── Route table ──────────────────────────────────────────────────────────
//
// `StaticSegment<T>` and `ParamSegment<T>` are generic over `T: AsPath`.
// Rather than threading raw `&'static str` literals through the route
// table, we define two closed enums and implement `AsPath` for each. That
// makes the set of static pieces and parameter names exhaustive and
// typo-proof, and lets the route consts read like the original `path!`
// invocations did.
//
// The `s` / `p` helpers are `const fn` constructors so the const route
// tuples stay short.

mod route;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang=HTML_LANG dir=HTML_DIR>
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
        <Stylesheet id="leptos" href=STYLESHEET_URL/>
        <Title text=APP_TITLE/>
        <Router>
            <Routes fallback=|| "Page not found.".into_view()>
                <Route path=route::LOGIN view={Lazy::<LoginPage>::new()}/>
                <ParentRoute path=route::PARENT view=Layout>
                    <Route path=route::ROOT       view={Lazy::<HomePage>::new()}/>
                    <Route path=route::section()    view={Lazy::<SectionListingPage>::new()}/>
                    <Route path=route::collection() view={Lazy::<CollectionDetailPage>::new()}/>
                    <Route path=route::item()       view={Lazy::<CollectionDetailPage>::new()}/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
