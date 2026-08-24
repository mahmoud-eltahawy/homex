use crate::app::common::{CardsLoading, MediaPageHeader};
use crate::app::model::Movie;
use crate::app::movies::fetch_movies;
use crate::app::view_schema::CardsList;
use crate::app::{icons::MovieIcon, resource_view::ResourceView};
use leptos::prelude::*;
use leptos_router::{lazy_route, LazyRoute};
pub struct MoviesPage {
    pub movies: Resource<Result<Vec<Movie>, ServerFnError>>,
}

#[lazy_route]
impl LazyRoute for MoviesPage {
    fn data() -> Self {
        let movies = Resource::new(|| (), async |_| fetch_movies(0, 20).await);
        Self { movies }
    }

    fn view(this: Self) -> AnyView {
        view! {
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <MediaPageHeader title="أفلام".to_string() icon=MovieIcon()/>
                <ResourceView
                    resource=this.movies
                    view_fn=CardsList::cards_list
                    adapter=|x| x
                    context="تحميل االفلام"
                    fallback=CardsLoading
                />
            </div>
        }
        .into_any()
    }
}
