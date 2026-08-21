use super::{model::Media, model::MediaType};
use crate::app::{
    common::{CardsLoading, MediaCard},
    icons::{MovieIcon, SeriesIcon},
    model::{Movie, Series},
    movies::listing::{fetch_movies, fetch_movies_count},
    resource_view::ResourceView,
    series::{fetch_series, fetch_series_count},
};
use leptos::prelude::*;
use leptos_router::{lazy_route, LazyRoute};

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
fn MediaSection(
    title: String,
    icon: impl IntoView,
    items: Vec<impl Into<Media>>,
    kind: MediaType,
    items_offset: RwSignal<usize>,
    items_count: Resource<Result<usize, ServerFnError>>,
) -> impl IntoView {
    let go_left = move |_| {
        if let Some(count) = items_count.get().transpose().ok().flatten() {
            items_offset.update(|x| {
                if *x < count - MEDIA_LIST_SIZE {
                    *x += 1
                }
            });
        };
    };
    let go_right = move |_| {
        items_offset.update(|x| {
            if *x > 0 {
                *x -= 1
            }
        });
    };
    view! {
        <section class="mb-12 md:mb-16">
            <button on:click=go_left>"left"</button>
            <div class="flex items-center justify-between mb-6">
                <h2 class="text-2xl sm:text-3xl md:text-4xl font-black text-white flex items-center gap-3">
                    <span class="text-cyan-400">{icon}</span> {title.clone()}
                </h2>
                <a
                    class="text-cyan-400 hover:text-cyan-300 text-sm font-medium transition-all flex items-center gap-1 group"
                    href={kind.to_string()}>
                    <span class="text-lg group-hover:translate-x-1 transition-transform">"←"</span> " عرض الكل"
                </a>
            </div>
            <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4 md:gap-6">
                {items.into_iter().map(|item| view! { <MediaCard item=item.into()/> }).collect_view()}
            </div>
            <button on:click=go_right>"right"</button>
        </section>
    }
}

pub struct HomePage {
    movies_offset: RwSignal<usize>,
    series_offset: RwSignal<usize>,
    series: Resource<Result<Vec<Series>, ServerFnError>>,
    movies: Resource<Result<Vec<Movie>, ServerFnError>>,
    movies_count: Resource<Result<usize, ServerFnError>>,
    series_count: Resource<Result<usize, ServerFnError>>,
}

const MEDIA_LIST_SIZE: usize = 5;

#[lazy_route]
impl LazyRoute for HomePage {
    fn data() -> Self {
        let movies_offset = RwSignal::new(0usize);
        let series_offset = RwSignal::new(0);
        let series = Resource::new(
            move || series_offset.get(),
            async |offset| fetch_series(offset, MEDIA_LIST_SIZE).await,
        );
        let movies = Resource::new(
            move || movies_offset.get(),
            async |offset| fetch_movies(offset, MEDIA_LIST_SIZE).await,
        );
        let movies_count = Resource::new(|| (), async |_| fetch_movies_count().await);
        let series_count = Resource::new(|| (), async |_| fetch_series_count().await);
        Self {
            movies_offset,
            movies,
            series_offset,
            series,
            movies_count,
            series_count,
        }
    }

    fn view(this: Self) -> AnyView {
        let movie_adapter = move |movies: Vec<Movie>| MediaSectionProps {
            title: "أفلام".to_string(),
            icon: MovieIcon(),
            items: movies,
            kind: MediaType::Movie,
            items_offset: this.movies_offset,
            items_count: this.movies_count,
        };
        let series_adapter = move |series: Vec<Series>| MediaSectionProps {
            title: "مسلسلات".to_string(),
            icon: SeriesIcon(),
            items: series,
            kind: MediaType::Series,
            items_offset: this.series_offset,
            items_count: this.series_count,
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
