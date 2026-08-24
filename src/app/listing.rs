use crate::app::{
    audio::fetch_audio_groups,
    common::{CardsLoading, MediaPageHeader},
    model::{AudioGroup, Movie, Series},
    movies::fetch_movies,
    resource_view::ResourceView,
    series::fetch_series,
    view_schema::{Card, CardsList},
};
use leptos::prelude::*;
use leptos_router::{lazy_route, LazyRoute};
use serde::{de::DeserializeOwned, Serialize};

pub struct ListingPage<C>
where
    C: Card + Send + Sync + Clone + Serialize + DeserializeOwned + 'static,
{
    pub data: Resource<Result<Vec<C>, ServerFnError>>,
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
            </div>
        }
    }
}

pub type SeriesListingPage = ListingPage<Series>;

#[lazy_route]
impl LazyRoute for SeriesListingPage {
    fn data() -> Self {
        Self {
            data: Resource::new(|| (), |_| fetch_series(0, 20)),
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
        let data = Resource::new(|| (), async |_| fetch_movies(0, 20).await);
        Self { data }
    }

    fn view(this: Self) -> AnyView {
        this.view().into_any()
    }
}

pub type AudioGroupListingPage = ListingPage<AudioGroup>;

#[lazy_route]
impl LazyRoute for AudioGroupListingPage {
    fn data() -> Self {
        let data = Resource::new(|| (), async |_| fetch_audio_groups(0, 20).await);
        Self { data }
    }

    fn view(this: Self) -> AnyView {
        this.view().into_any()
    }
}
