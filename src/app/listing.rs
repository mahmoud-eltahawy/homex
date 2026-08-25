use crate::app::pagination::PaginationControls;
use crate::app::{
    audio::{fetch_audio_groups, fetch_audio_groups_count},
    common::CardsLoading,
    model::{AudioGroup, Movie, Series},
    movies::{fetch_movies, fetch_movies_count},
    resource_view::ResourceView,
    series::{fetch_series, fetch_series_count},
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

pub type SeriesListingPage = ListingPage<Series>;

#[lazy_route]
impl LazyRoute for SeriesListingPage {
    fn data() -> Self {
        let offset = RwSignal::new(0);
        let data = Resource::new(
            move || offset.get(),
            |offset| fetch_series(offset, LISTING_PAGE_SIZE),
        );
        let count = Resource::new(|| (), |_| fetch_series_count());
        Self {
            data,
            offset,
            count,
        }
    }

    fn view(this: Self) -> AnyView {
        this.view().into_any()
    }
}

pub type MovieListingPage = ListingPage<Movie>;

#[lazy_route]
impl LazyRoute for MovieListingPage {
    fn data() -> Self {
        let offset = RwSignal::new(0);
        let data = Resource::new(
            move || offset.get(),
            |offset| fetch_movies(offset, LISTING_PAGE_SIZE),
        );
        let count = Resource::new(|| (), |_| fetch_movies_count());
        Self {
            data,
            offset,
            count,
        }
    }

    fn view(this: Self) -> AnyView {
        this.view().into_any()
    }
}

pub type AudioGroupListingPage = ListingPage<AudioGroup>;

#[lazy_route]
impl LazyRoute for AudioGroupListingPage {
    fn data() -> Self {
        let offset = RwSignal::new(0);
        let data = Resource::new(
            move || offset.get(),
            |offset| fetch_audio_groups(offset, LISTING_PAGE_SIZE),
        );
        let count = Resource::new(|| (), |_| fetch_audio_groups_count());
        Self {
            data,
            offset,
            count,
        }
    }

    fn view(this: Self) -> AnyView {
        this.view().into_any()
    }
}
