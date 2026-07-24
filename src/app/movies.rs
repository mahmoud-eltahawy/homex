pub mod listing {
    use crate::app::common::{CardsLoading, MediaCard, MediaCardProps, MediaPageHeader};
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
            let movies = Resource::new(|| (), |_| async move { fetch_movies().await });
            Self { movies }
        }

        fn view(this: Self) -> AnyView {
            let adapter = move |x| MoviesCardsProps { movies: x };
            view! {
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                    <MediaPageHeader title="أفلام".to_string() icon=MovieIcon()/>
                    <Suspense fallback=CardsLoading>
                        <ResourceView
                            resource=this.movies
                            view_fn=MoviesCards
                            adapter=adapter
                            context="تحميل االفلام"
                        />
                    </Suspense>
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
                    .map(|item| MediaCard(MediaCardProps { item : item.into() }))
                    .collect_view()
                }
            </div>
        }
    }

    #[server]
    pub async fn fetch_movies() -> Result<Vec<model::Movie>, ServerFnError> {
        use crate::app::delay;
        use crate::app::mockary;
        delay(300).await;
        Ok(mockary::mock_movies())
    }
}

pub mod detail;
