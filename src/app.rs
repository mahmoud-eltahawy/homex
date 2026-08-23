use crate::app::{
    home::HomePage,
    layout::Layout,
    movies::{detail::MovieDetailPage, listing::MoviesPage},
    series::{details::SeriesDetailPage, listing::SeriesPage},
    settings::SettingsPage,
    upload::UploadPage,
};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{ParentRoute, Route, Router, Routes},
    path, Lazy,
};

pub trait IconView {
    fn icon() -> impl IntoView;
}

pub trait PosterSvgView {
    fn svg_poster() -> impl IntoView;
}

pub trait PosterView: PosterSvgView {
    fn poster(self) -> impl IntoView;
}

pub trait OverPosterView: PosterSvgView {
    fn over_poster(self) -> impl IntoView;
}

pub trait CardImageView: IconView + PosterSvgView + OverPosterView {
    fn card_image(self) -> impl IntoView;
}

pub trait InfoView {
    fn info_view(self) -> impl IntoView;
}

pub trait Href {
    fn href(self) -> String;
}

pub trait Card: Href + CardImageView + InfoView {
    fn card(self) -> impl IntoView;
}

impl<T> Card for T
where
    T: Href + CardImageView + InfoView + Clone,
{
    fn card(self) -> impl IntoView {
        let href = self.clone().href();
        view! {
            <a href=href class="group relative flex flex-col overflow-hidden rounded-2xl bg-[#1a1a24]/80 backdrop-blur-sm border border-white/5 shadow-2xl hover:shadow-cyan-500/20 transition-all duration-500 hover:scale-[1.03] hover:-translate-y-2">
                {self.clone().card_image()}
                {self.info_view()}
            </a>
        }
    }
}

pub trait CardsList {
    fn cards_list(self) -> impl IntoView;
}

impl<T> CardsList for Vec<T>
where
    T: Card,
{
    fn cards_list(self) -> impl IntoView {
        view! {
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6">
                {
                self
                    .into_iter()
                    .map(|item| item.card())
                    .collect_view()
                }
            </div>
        }
    }
}

mod common;
mod home;
mod icons;
mod layout;
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

mod audio;

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
                    <Route path=path!("/movie") view={Lazy::<MoviesPage>::new()}/>
                    <Route path=path!("/series") view={Lazy::<SeriesPage>::new()}/>
                    <Route path=path!("/upload") view={Lazy::<UploadPage>::new()}/>
                    <Route path=path!("/settings") view={Lazy::<SettingsPage>::new()}/>
                    <Route path=path!("/detail/series/:id") view={Lazy::<SeriesDetailPage>::new()}/>
                    <Route path=path!("/detail/movie/:id") view={Lazy::<MovieDetailPage>::new()}/>
                </ParentRoute>
            </Routes>
        </Router>
    }
}
