use super::{Media, MediaType};
use crate::app::{
    fetch_movies,
    icons::{MovieIcon, SeriesIcon},
    resource_view::ResourceView,
    series::{fetch_series, listing::Series},
    CardsLoading, MediaCard, Movie,
};
use leptos::prelude::*;
use leptos_router::{hooks::use_navigate, lazy_route, LazyRoute};
use web_sys::MouseEvent;

#[component]
pub fn HomeHero() -> impl IntoView {
    view! {
        <div class="py-12 sm:py-16 md:py-20 lg:py-24 text-center">
            <h1 class="text-4xl sm:text-5xl md:text-6xl lg:text-7xl font-black tracking-tight leading-[1.1]">
                <span class="bg-gradient-to-r from-cyan-200 via-blue-300 to-indigo-400 bg-clip-text text-transparent">"سينماك"</span>
                <br class="sm:hidden"/>
                <span class="text-white">" الشخصية"</span>
            </h1>
            <p class="text-gray-400 text-base sm:text-lg md:text-xl max-w-2xl mx-auto mt-4 leading-relaxed">
                "شاهد وحمّل مجموعتك من الأفلام والمسلسلات من أي مكان في منزلك."
            </p>
        </div>
    }
}

#[component]
pub fn MediaSection(
    title: String,
    icon: impl IntoView,
    items: Vec<Media>,
    kind: MediaType,
) -> impl IntoView {
    let navigate = use_navigate();
    let on_click = move |ev: MouseEvent| {
        ev.prevent_default();
        navigate(&kind.to_string(), Default::default());
    };
    view! {
        <section class="mb-12 md:mb-16">
            <div class="flex items-center justify-between mb-6">
                <h2 class="text-2xl sm:text-3xl md:text-4xl font-black text-white flex items-center gap-3">
                    <span class="text-cyan-400">{icon}</span> {title.clone()}
                </h2>
                <a class="text-cyan-400 hover:text-cyan-300 text-sm font-medium transition-all flex items-center gap-1 group"
                    on:click=on_click>
                    <span class="text-lg group-hover:translate-x-1 transition-transform">"←"</span> " عرض الكل"
                </a>
            </div>
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4 md:gap-6">
                {items.into_iter().take(5).map(|item| view! { <MediaCard item=item/> }).collect_view()}
            </div>
        </section>
    }
}

pub struct HomePage {
    series: Resource<Result<Vec<Series>, ServerFnError>>,
    movies: Resource<Result<Vec<Movie>, ServerFnError>>,
}

#[lazy_route]
impl LazyRoute for HomePage {
    fn data() -> Self {
        let series = Resource::new(|| (), |_| async move { fetch_series().await });
        let movies = Resource::new(|| (), |_| async move { fetch_movies().await });
        Self { series, movies }
    }

    fn view(this: Self) -> AnyView {
        let movie_adapter = move |movies: Vec<Movie>| MediaSectionProps {
            title: "أفلام".to_string(),
            icon: MovieIcon(),
            items: movies.into_iter().map(Media::Movie).collect(),
            kind: MediaType::Movie,
        };
        let series_adapter = move |series: Vec<Series>| MediaSectionProps {
            title: "مسلسلات".to_string(),
            icon: SeriesIcon(),
            items: series.into_iter().map(Media::Series).collect(),
            kind: MediaType::Series,
        };
        view! {
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <HomeHero/>
                <ResourceView
                    resource=this.movies
                    view_fn=MediaSection
                    adapter=movie_adapter
                    fallback=CardsLoading
                    context="تحميل الافلام"
                />
                <ResourceView
                    resource=this.series
                    view_fn=MediaSection
                    adapter=series_adapter
                    fallback=CardsLoading
                    context="تحميل مسلسلات"
                />
            </div>
        }
        .into_any()
    }
}
