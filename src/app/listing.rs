use std::future::Future;

use crate::app::pagination::PaginationControls;
use crate::app::series::fetch_series;
use crate::app::{
    audio::{fetch_audio_groups, fetch_audio_groups_count},
    common::CardsLoading,
    model::{AudioGroup, Movie, Series},
    movies::{fetch_movies, fetch_movies_count},
    resource_view::ResourceView,
    series::fetch_series_count,
    view_schema::{Card, CardsList},
};
use leptos::prelude::*;
use leptos_router::{lazy_route, LazyRoute};
use serde::{de::DeserializeOwned, Serialize};

const LISTING_PAGE_SIZE: usize = 18;

pub struct ListingPage<C>
where
    C: Card + Send + Sync + Clone + Serialize + DeserializeOwned + 'static,
{
    pub offset: RwSignal<usize>,
    pub search_query: RwSignal<Option<String>>,
    pub data: Resource<Result<Vec<C>, ServerFnError>>,
    pub count: Resource<Result<usize, ServerFnError>>,
}

impl<C> ListingPage<C>
where
    C: Card + Send + Sync + Clone + Serialize + DeserializeOwned + 'static,
{
    fn view(self) -> impl IntoView {
        let title = C::media_type().ar_title();
        let context = format!("تحميل {} ...", title);
        view! {
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <MediaPageHeader
                    title=title.to_string()
                    icon=C::icon()
                    offset=self.offset
                    search_query=self.search_query
                />
                <ResourceView
                    resource=self.data
                    view_fn=CardsList::cards_list
                    fallback=CardsLoading
                    adapter=|x| x
                    context=context
                />
                <PaginationControls
                    offset=self.offset
                    count=self.count
                    page_size=LISTING_PAGE_SIZE
                />
            </div>
        }
    }
}

#[component]
fn MediaPageHeader(
    title: String,
    icon: impl IntoView,
    offset: RwSignal<usize>,
    search_query: RwSignal<Option<String>>,
) -> impl IntoView {
    let on_search = move |ev| {
        let query = event_target_value(&ev);
        batch(move || {
            offset.set(0);
            search_query.set(if query.is_empty() { None } else { Some(query) });
        });
    };

    view! {
        <div class="flex flex-col gap-4 mb-6 md:mb-8">
            <div class="flex items-center gap-4">
                <div class="p-3 bg-cyan-400/10 rounded-2xl text-cyan-400">{icon}</div>
                <div>
                    <h1 class="text-3xl sm:text-4xl md:text-5xl font-black text-white">{title.clone()}</h1>
                    <p class="text-gray-400 text-sm md:text-base mt-0.5">
                        "تصفح مجموعة "{title}"ك"
                    </p>
                </div>
            </div>
            <div class="relative">
                <input
                    type="text"
                    placeholder="ابحث..."
                    class="w-full bg-gray-800 text-white rounded-xl px-4 py-3 pr-10 outline-none focus:ring-2 focus:ring-cyan-400"
                    prop:value=move || search_query.get().unwrap_or_default()
                    on:change=on_search
                />
                // Optional search icon
                <span class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">
                    // SVG or similar
                </span>
            </div>
        </div>
    }
}

impl<T> ListingPage<T>
where
    T: Card + Send + Sync + Clone + Serialize + DeserializeOwned + 'static,
{
    fn new<Fut1, Fut2>(
        data_fn: impl Fn(usize, usize, Option<String>) -> Fut1 + Send + Sync + 'static,
        count_fn: impl Fn(Option<String>) -> Fut2 + Send + Sync + 'static,
    ) -> Self
    where
        Fut1: Future<Output = Result<Vec<T>, ServerFnError>> + Send + 'static,
        Fut2: Future<Output = Result<usize, ServerFnError>> + Send + 'static,
    {
        let offset = RwSignal::new(0);
        let search_query = RwSignal::new(None);
        let data = Resource::new(
            move || (offset.get(), search_query.get()),
            move |(offset, search_query)| data_fn(offset, LISTING_PAGE_SIZE, search_query),
        );
        let count = Resource::new(move || search_query.get(), count_fn);
        Self {
            offset,
            search_query,
            data,
            count,
        }
    }
}

pub type SeriesListingPage = ListingPage<Series>;

#[lazy_route]
impl LazyRoute for SeriesListingPage {
    fn data() -> Self {
        Self::new(fetch_series, fetch_series_count)
    }

    fn view(this: Self) -> AnyView {
        this.view().into_any()
    }
}

pub type MovieListingPage = ListingPage<Movie>;

#[lazy_route]
impl LazyRoute for MovieListingPage {
    fn data() -> Self {
        Self::new(fetch_movies, fetch_movies_count)
    }

    fn view(this: Self) -> AnyView {
        this.view().into_any()
    }
}

pub type AudioGroupListingPage = ListingPage<AudioGroup>;

#[lazy_route]
impl LazyRoute for AudioGroupListingPage {
    fn data() -> Self {
        Self::new(fetch_audio_groups, fetch_audio_groups_count)
    }

    fn view(this: Self) -> AnyView {
        this.view().into_any()
    }
}
