use super::{model::Media, model::MediaType};
use crate::app::{
    common::{CardsLoading, MediaCard},
    icons::{MovieIcon, NextIcon, PrevIcon, SeriesIcon},
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
    let can_prev = move || items_offset.get() > 0;

    let can_next = move || {
        items_count
            .get()
            .transpose()
            .ok()
            .flatten()
            .map(|count| items_offset.get() < count.saturating_sub(MEDIA_LIST_SIZE))
            .unwrap_or(false)
    };

    let go_prev = move |_| {
        items_offset.update(|x| {
            if *x > 0 {
                *x -= 1;
            }
        });
    };

    let go_next = move |_| {
        items_offset.update(|x| {
            let max_offset = items_count
                .get()
                .transpose()
                .ok()
                .flatten()
                .unwrap_or(0)
                .saturating_sub(MEDIA_LIST_SIZE);

            if *x < max_offset {
                *x += 1;
            }
        });
    };

    view! {
        <section class="mb-12 md:mb-16">
            <div class="mb-6 flex flex-wrap items-center justify-between gap-4">
                <h2 class="flex items-center gap-3 text-2xl font-black text-white sm:text-3xl md:text-4xl">
                    <span class="text-cyan-400">{icon}</span>
                    {title.clone()}
                </h2>

                <div class="flex items-center gap-3 sm:gap-4">
                    <div class="flex items-center gap-1 rounded-full border border-white/10 bg-white/5 p-1 backdrop-blur">
                        <button
                            on:click=go_prev
                            disabled=move || !can_prev()
                            aria-label="Previous"
                            class="flex h-8 w-8 items-center justify-center rounded-full text-slate-300 transition hover:bg-white/10 hover:text-white focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 disabled:cursor-not-allowed disabled:opacity-30"
                        >
                            <PrevIcon/>
                        </button>

                        <button
                            on:click=go_next
                            disabled=move || !can_next()
                            aria-label="Next"
                            class="flex h-8 w-8 items-center justify-center rounded-full text-slate-300 transition hover:bg-white/10 hover:text-white focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 disabled:cursor-not-allowed disabled:opacity-30"
                        >
                            <NextIcon/>
                        </button>
                    </div>

                    <a
                        href={kind.to_string()}
                        class="group inline-flex items-center gap-1 text-sm font-semibold text-cyan-400 transition hover:text-cyan-300"
                    >
                        <span class="transition-transform group-hover:translate-x-1">"عرض الكل"</span>
                        <span class="text-lg transition-transform group-hover:translate-x-1" aria-hidden="true">"←"</span>
                    </a>
                </div>
            </div>

            <div class="grid grid-cols-2 gap-4 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 md:gap-6">
                {items.into_iter().map(|item| view! { <MediaCard item=item.into()/> }).collect_view()}
            </div>
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
