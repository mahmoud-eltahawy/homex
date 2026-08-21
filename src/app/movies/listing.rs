use crate::app::common::{CardsLoading, MediaPageHeader, MovieCard, MovieCardProps};
use crate::app::model::{self, Movie};
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
        let adapter = move |x| MoviesCardsProps { movies: x };
        view! {
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                <MediaPageHeader title="أفلام".to_string() icon=MovieIcon()/>
                <ResourceView
                    resource=this.movies
                    view_fn=MoviesCards
                    adapter=adapter
                    context="تحميل االفلام"
                    fallback=CardsLoading
                />
            </div>
        }
        .into_any()
    }
}

#[component]
pub fn MoviesCards(movies: Vec<Movie>) -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6">
            {
            movies
                .into_iter()
                .map(|item| MovieCard(MovieCardProps{ item  }))
                .collect_view()
            }
        </div>
    }
}

#[server]
pub async fn fetch_movies(offset: usize, size: usize) -> Result<Vec<model::Movie>, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary;
    delay(300).await;
    let list = mockary::mock_movies();
    let size = size.clamp(0, list.len());
    let offset = offset.clamp(0, list.len() - size);
    let end = (offset + size).clamp(0, list.len());

    Ok(list[offset..end].to_vec())
}

#[server]
pub async fn fetch_movies_count() -> Result<usize, ServerFnError> {
    use crate::app::delay;
    use crate::app::mockary;
    delay(300).await;

    Ok(mockary::mock_movies().len())
}
