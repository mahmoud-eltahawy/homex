use std::future::Future;

use crate::app::{
    audio::{fetch_audio_groups, fetch_audio_groups_count},
    common::CardsLoading,
    icons::{EmptyStateIcon, NextPageIcon, PrevPageIcon, ViewAllIcon},
    model::{AudioGroup, Movie, Series},
    movies::{fetch_movies, fetch_movies_count},
    resource_view::ResourceView,
    series::{fetch_series, fetch_series_count},
    view_schema::{Card, CardsList},
};
use leptos::{either::Either, prelude::*};
use leptos_router::{lazy_route, LazyRoute};
use serde::{de::DeserializeOwned, Serialize};

impl<T> MediaLoaderProps<T>
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
        let offset = RwSignal::new(0usize);
        let resource = Resource::new(
            move || offset.get(),
            move |offset| data_fn(offset, MEDIA_LIST_SIZE, None),
        );
        let count = Resource::new(|| None, count_fn);
        Self {
            offset,
            resource,
            count,
        }
    }
}

pub struct HomePage {
    movies: MediaLoaderProps<Movie>,
    series: MediaLoaderProps<Series>,
    audio: MediaLoaderProps<AudioGroup>,
}

const MEDIA_LIST_SIZE: usize = 6;

#[lazy_route]
impl LazyRoute for HomePage {
    fn data() -> Self {
        let movies = MediaLoaderProps::new(fetch_movies, fetch_movies_count);
        let series = MediaLoaderProps::new(fetch_series, fetch_series_count);
        let audio = MediaLoaderProps::new(fetch_audio_groups, fetch_audio_groups_count);

        Self {
            movies,
            series,
            audio,
        }
    }

    fn view(this: Self) -> AnyView {
        let HomePage {
            movies,
            series,
            audio,
        } = this;
        view! {
            <div class="min-h-screen bg-[#0c0b1a] text-white">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 md:py-10">
                    {MediaLoader(movies)}
                    <hr class="border-t border-white/5 my-10 md:my-12" />
                    {MediaLoader(series)}
                    <hr class="border-t border-white/5 my-10 md:my-12" />
                    {MediaLoader(audio)}
                </div>
            </div>
        }
        .into_any()
    }
}

#[component]
fn MediaSection<C>(
    items: Vec<C>,
    items_offset: RwSignal<usize>,
    items_count: Resource<Result<usize, ServerFnError>>,
) -> impl IntoView
where
    C: Card + 'static,
{
    let media_type = C::media_type();

    let header_adapter = move |count| SectionHeaderProps {
        icon: C::icon(),
        count,
        href: media_type.listing_href(),
    };
    let pagination_adapter = move |count| SectionPaginationProps {
        offset: items_offset,
        total_count: count,
        page_size: MEDIA_LIST_SIZE,
    };
    view! {
        <section class="bg-white/5 rounded-2xl p-4 md:p-6">
            <ResourceView
                resource=items_count
                view_fn=SectionHeader
                adapter=header_adapter
                context=""
            />
            <SectionContent items={items} />
            <ResourceView
                resource=items_count
                view_fn=SectionPagination
                adapter=pagination_adapter
                context=""
            />
        </section>
    }
}

// ─── Header: Icon + Count + ViewAll ────────────────────────────────────────

#[component]
fn SectionHeader(icon: impl IntoView + 'static, count: usize, href: String) -> impl IntoView {
    view! {
        <div class="flex items-center justify-between mb-6">
            <div class="flex items-center gap-3">
                <span class="flex items-center">{icon}</span>
                <span class="text-sm font-mono text-white/60 bg-white/10 px-3 py-0.5 rounded-full">
                    {count}
                </span>
            </div>
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

// ─── Content: Grid or Empty State ──────────────────────────────────────────

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

// ─── Pagination (Bottom) ────────────────────────────────────────────────────

#[component]
fn SectionPagination(
    offset: RwSignal<usize>,
    total_count: usize,
    page_size: usize,
) -> impl IntoView {
    let total_pages = total_count.div_ceil(page_size).max(1);

    if total_pages <= 1 {
        return Either::Left(view! { <div class="h-1" /> });
    }

    let can_prev = move || offset.get() == 0;
    let can_next = move || offset.get() > total_count.saturating_sub(page_size);

    let go_prev = move |_| {
        offset.update(|x| {
            if *x > 0 {
                *x -= 1
            }
        });
    };
    let go_next = move |_| {
        offset.update(|x| {
            let max = total_count.saturating_sub(page_size);
            if *x < max {
                *x += 1;
            }
        });
    };

    let status = move || {
        let current = offset.get() + 1;
        format!("{} / {}", current, total_pages)
    };

    Either::Right(view! {
        <div class="flex items-center justify-end gap-4 mt-6 pt-4 border-t border-white/5">
            <div class="flex items-center gap-2">
                <button
                    on:click=go_prev
                    disabled=can_prev
                    class="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-white/10 disabled:opacity-30 disabled:pointer-events-none transition-colors"
                    aria-label="Previous page"
                >
                    <PrevPageIcon />
                </button>

                <span class="font-mono text-sm text-white/60 px-3 py-1 bg-white/10 rounded-lg min-w-[3.5rem] text-center">
                    {status}
                </span>

                <button
                    on:click=go_next
                    disabled=can_next
                    class="p-2 rounded-lg text-gray-400 hover:text-white hover:bg-white/10 disabled:opacity-30 disabled:pointer-events-none transition-colors"
                    aria-label="Next page"
                >
                    <NextPageIcon />
                </button>
            </div>
        </div>
    })
}

#[component]
fn MediaLoader<T>(
    resource: Resource<Result<Vec<T>, ServerFnError>>,
    offset: RwSignal<usize>,
    count: Resource<Result<usize, ServerFnError>>,
) -> impl IntoView
where
    T: Card + Send + Sync + Serialize + DeserializeOwned + Clone + 'static,
{
    let context = format!("تحميل {}...", T::media_type().ar_title());

    let adapter = move |items: Vec<T>| MediaSectionProps {
        items,
        items_offset: offset,
        items_count: count,
    };

    view! {
        <ResourceView
            resource={resource}
            view_fn={MediaSection}
            adapter={adapter}
            fallback={CardsLoading}
            context={context}
        />
    }
}
