use crate::app::search::SearchBar;
use std::future::Future;

use crate::app::{
    audio::{fetch_audio_groups, fetch_audio_groups_count},
    icons::{EmptyStateIcon, ViewAllIcon},
    model::{AudioGroup, Movie, Series},
    movies::{fetch_movies, fetch_movies_count},
    pagination::{PaginationControls, PaginationControlsProps},
    resource_view::ResourceView,
    series::{fetch_series, fetch_series_count},
    view_schema::{Card, CardsList},
};
use leptos::{either::Either, prelude::*};
use leptos_router::{lazy_route, LazyRoute};
use serde::{de::DeserializeOwned, Serialize};

impl<T> MediaSectionProps<T>
where
    T: Card + Send + Sync + Clone + Serialize + DeserializeOwned + 'static,
{
    fn new<Fut1, Fut2>(
        search_query: RwSignal<Option<String>>,
        data_fn: impl Fn(usize, usize, Option<String>) -> Fut1 + Send + Sync + Copy + 'static,
        count_fn: impl Fn(Option<String>) -> Fut2 + Send + Sync + 'static,
    ) -> Self
    where
        Fut1: Future<Output = Result<Vec<T>, ServerFnError>> + Send + 'static,
        Fut2: Future<Output = Result<usize, ServerFnError>> + Send + 'static,
    {
        let folded = RwSignal::new(false);
        let offset = RwSignal::new(0usize);
        let resource_trigger = move || {
            if !folded.get() {
                (false, offset.get(), search_query.get())
            } else {
                (true, 0, None)
            }
        };

        let items = Resource::new(
            resource_trigger,
            move |(folded, offset, search_query)| async move {
                if !folded {
                    data_fn(offset, MEDIA_LIST_SIZE, search_query).await
                } else {
                    Ok(Vec::new())
                }
            },
        );
        let count = Resource::new(move || search_query.get(), count_fn);
        Self {
            folded,
            offset,
            items,
            count,
        }
    }
}

pub struct HomePage {
    search_query: RwSignal<Option<String>>,
    movies: MediaSectionProps<Movie>,
    series: MediaSectionProps<Series>,
    audio: MediaSectionProps<AudioGroup>,
}

const MEDIA_LIST_SIZE: usize = 6;

#[lazy_route]
impl LazyRoute for HomePage {
    fn data() -> Self {
        let search_query = RwSignal::new(None);
        let movies = MediaSectionProps::new(search_query, fetch_movies, fetch_movies_count);
        let series = MediaSectionProps::new(search_query, fetch_series, fetch_series_count);
        let audio =
            MediaSectionProps::new(search_query, fetch_audio_groups, fetch_audio_groups_count);

        Self {
            search_query,
            movies,
            series,
            audio,
        }
    }

    fn view(this: Self) -> AnyView {
        let HomePage {
            search_query,
            movies,
            series,
            audio,
        } = this;
        let offset_reset = move || {
            movies.offset.set(0);
            series.offset.set(0);
            audio.offset.set(0);
        };
        view! {
            <div class="min-h-screen bg-[#0c0b1a] text-white">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 md:py-10 flex flex-col">
                    <SearchBar
                        search_query
                        offset_reset
                    />
                    {MediaSection(movies)}
                    {MediaSection(series)}
                    {MediaSection(audio)}
                </div>
            </div>
        }
        .into_any()
    }
}

#[component]
fn MediaSection<C>(
    folded: RwSignal<bool>,
    items: Resource<Result<Vec<C>, ServerFnError>>,
    offset: RwSignal<usize>,
    count: Resource<Result<usize, ServerFnError>>,
) -> impl IntoView
where
    C: Card + Send + Sync + Clone + Serialize + DeserializeOwned + 'static,
{
    let media_type = C::media_type();

    let header_adapter = move |count| SectionHeaderProps {
        folded,
        icon: C::icon(),
        count,
        href: media_type.listing_href(),
    };
    let pagination_adapter = move |count| PaginationControlsProps {
        offset,
        page_size: MEDIA_LIST_SIZE,
        count,
        window_size: 5,
    };
    let content_adapter = move |items| SectionContentProps { items };
    let order = move || if folded.get() { "1" } else { "0" };
    view! {
        <div style:order=order >
            <hr class="border-t border-white/5 my-10 md:my-12" />
            <section class="bg-white/5 rounded-2xl p-4 md:p-6">
                <ResourceView
                    resource=count
                    view_fn=SectionHeader
                    adapter=header_adapter
                />
                <Show when=move || !folded.get()>
                    <ResourceView
                        resource=items
                        view_fn=SectionContent
                        adapter=content_adapter
                    />
                    <ResourceView
                        resource=count
                        view_fn=PaginationControls
                        adapter=pagination_adapter
                    />
                </Show>
            </section>
        </div>
    }
}

#[component]
fn SectionHeader(
    icon: impl IntoView + 'static,
    count: usize,
    href: String,
    folded: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-3">
                <span class="flex items-center">{icon}</span>
                <span class="text-sm font-mono text-white/60 bg-white/10 px-3 py-0.5 rounded-full">
                    {count}
                </span>
            </div>
            <FoldButton folded/>
            <a
                href=href
                class="group p-1 rounded hover:bg-white/10 transition-colors"
                aria-label="View all"
            >
                <ViewAllIcon />
            </a>
        </div>
    }
}

#[component]
fn FoldButton(folded: RwSignal<bool>) -> impl IntoView {
    let on_click = move |_| folded.update(|x| *x = !*x);
    view! {
        <button
            on:click=on_click
            class="p-1 rounded hover:bg-white/10 transition-colors"
            aria-label="Toggle section"
        >
            <svg
                class="w-5 h-5 transition-transform duration-200"
                style=move || format!("transform: rotate({}deg)", if folded.get() { 180 } else { 0 })
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
                stroke-width="2"
            >
                <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
            </svg>
        </button>
    }
}

#[component]
fn SectionContent<C: Card + 'static>(items: Vec<C>) -> impl IntoView {
    if !items.is_empty() {
        return Either::Right(items.cards_list());
    };
    Either::Left(view! {
        <div class="flex flex-col items-center justify-center py-12 gap-3">
            <EmptyStateIcon />
            <span class="text-white/20 text-sm font-mono">0</span>
        </div>
    })
}
