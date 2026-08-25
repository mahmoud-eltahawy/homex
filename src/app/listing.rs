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
                <MediaPageHeader title=title.to_string() icon=C::icon()/>
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
fn MediaPageHeader(title: String, icon: impl IntoView) -> impl IntoView {
    view! {
        <div class="flex items-center gap-4 mb-6 md:mb-8">
            <div class="p-3 bg-cyan-400/10 rounded-2xl text-cyan-400">{icon}</div>
            <div>
                <h1 class="text-3xl sm:text-4xl md:text-5xl font-black text-white">{title.clone()}</h1>
                <p class="text-gray-400 text-sm md:text-base mt-0.5">"تصفح مجموعة "{title}"ك"</p>
            </div>
        </div>
    }
}

impl<T> ListingPage<T>
where
    T: Card + Send + Sync + Clone + Serialize + DeserializeOwned + 'static,
{
    fn new<Fut1, Fut2>(
        f1: impl Fn(usize, usize, Option<String>) -> Fut1 + Send + Sync + 'static,
        f2: impl Fn() -> Fut2 + Send + Sync + 'static,
    ) -> Self
    where
        Fut1: Future<Output = Result<Vec<T>, ServerFnError>> + Send + 'static,
        Fut2: Future<Output = Result<usize, ServerFnError>> + Send + 'static,
    {
        let offset = RwSignal::new(0);
        let search_query = RwSignal::new(None);
        let data = Resource::new(
            move || (offset.get(), search_query.get()),
            move |(offset, search_query)| f1(offset, LISTING_PAGE_SIZE, search_query),
        );
        let count = Resource::new(|| (), move |_| f2());
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
