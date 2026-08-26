use crate::app::pagination::{PaginationControls, PaginationControlsProps};
use crate::app::search::SearchBar;
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
use std::future::Future;

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
        let adapter = move |count| PaginationControlsProps {
            offset: self.offset,
            count,
            window_size: 8,
            page_size: LISTING_PAGE_SIZE,
        };
        view! {
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                    <SearchBar search_query=self.search_query offset_reset=move || self.offset.set(0)/>
                <ResourceView
                    resource=self.data
                    view_fn=CardsList::cards_list
                    fallback=CardsLoading
                    adapter=|x| x
                />
                <ResourceView
                    resource=self.count
                    view_fn=PaginationControls
                    adapter=adapter
                />
            </div>
        }
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
