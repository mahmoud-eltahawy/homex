use crate::app::collections::{fetch_collections, fetch_collections_count};
use crate::app::error::AppError;
use crate::app::icons::EmptyStateIcon;
use crate::app::ids::Slug;
use crate::app::model::{Collection, SearchQuery};
use crate::app::tags::TagList;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use serde::Serialize;
use serde::de::DeserializeOwned;

#[component]
pub fn CardsLoading() -> impl IntoView {
    let cards = (0..5).map(|_| CardSkeleton()).collect_view();
    view! {
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6 my-15">
            {cards}
        </div>
    }
}

#[component]
pub fn CardSkeleton() -> impl IntoView {
    view! {
        <div class="animate-pulse rounded-2xl bg-[#1a1a24]/60 border border-white/5 overflow-hidden shadow-xl">
            <div class="aspect-[2/3] bg-gradient-to-b from-[#2a2a3a] to-[#1a1a24]"></div>
            <div class="p-4 space-y-2">
                <div class="h-3 bg-[#2a2a3a] rounded w-3/4"></div>
                <div class="h-2 bg-[#2a2a3a] rounded w-1/2"></div>
            </div>
        </div>
    }
}

#[component]
pub fn CollectionCard(collection: Collection) -> impl IntoView {
    let href = collection.href();
    let title = collection.title.clone();
    let poster = collection.poster.clone();
    let count = collection.items_count;
    let tags = collection.tags.clone();

    view! {
        <a
            href=href
            class="group rounded-2xl overflow-hidden bg-[#1a1a24]/80 backdrop-blur-sm border border-white/5 hover:border-cyan-400/40 shadow-2xl hover:shadow-cyan-500/20 transition-all duration-500 hover:scale-[1.03]"
        >
            <div class="aspect-[2/3] relative overflow-hidden bg-white/5">
                {poster.map(|p| view! {
                    <img
                        src=p
                        class="w-full h-full object-cover transition-transform duration-700 group-hover:scale-110"
                        loading="lazy"
                    />
                })}
                <div class="absolute bottom-2 end-2 bg-black/70 backdrop-blur-md rounded-full px-2 py-0.5 text-xs font-bold text-white border border-white/10">
                    {count}
                </div>
            </div>
            <div class="p-3 space-y-1.5">
                <div class="text-sm text-white font-semibold truncate">{title}</div>
                <TagList tags=tags max=2/>
            </div>
        </a>
    }
}

#[component]
pub fn CollectionGrid(collections: Vec<Collection>) -> impl IntoView {
    let cards = collections
        .into_iter()
        .map(|c| view! { <CollectionCard collection=c/> })
        .collect_view();

    view! {
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6">
            {cards}
        </div>
    }
}

#[component]
pub fn EmptyState(#[prop(into)] label: String) -> impl IntoView {
    view! {
        <div class="flex flex-col items-center justify-center py-12 gap-3">
            <EmptyStateIcon />
            <span class="text-white/40 text-sm">{label}</span>
        </div>
    }
}

/// Replace the current view's subtree with a brief "Redirecting…"
/// placeholder and schedule a navigation to `/error`.
///
/// Rendered by [`Page::into_view`](crate::app::page::Page::into_view) when
/// a URL parameter failed to parse. The redirect happens inside an
/// `Effect`, so the placeholder paints for a frame before the router
/// swaps the page.
#[component]
pub fn RedirectToError() -> impl IntoView {
    let navigate = use_navigate();
    Effect::new(move |_| {
        navigate("/error", Default::default());
    });
    view! {
        <div class="min-h-[40vh] flex items-center justify-center text-gray-500 text-sm">
            "Redirecting…"
        </div>
    }
}

pub fn refetch_on_success<V, A, R>(action: Action<A, Result<V, AppError>>, resource: Resource<R>)
where
    A: Send + Sync + 'static + Clone,
    V: Send + Sync + 'static + Clone,
    R: Serialize + DeserializeOwned + Send + Sync + 'static,
{
    Effect::new(move |_| {
        if action.value().get().is_some_and(|r| r.is_ok()) {
            resource.refetch();
        }
    });
}

pub type CollectionPage = Resource<Result<Vec<Collection>, AppError>>;
pub type CollectionCount = Resource<Result<usize, AppError>>;

/// Paginated collections for a section, keyed on the section's `Slug`.
///
/// Callers supply a `Slug` — never a `String`. The parse happens upstream
/// in `PageContent::load`; this function's contract is that its input is
/// already a concrete domain value.
pub fn collections_paginated(
    slug: impl Fn() -> Slug + Clone + Send + Sync + 'static,
    offset: RwSignal<usize>,
    search: RwSignal<Option<String>>,
    skip: impl Fn() -> bool + Copy + Send + Sync + 'static,
    page_size: usize,
) -> (CollectionPage, CollectionCount) {
    let slug_items = slug.clone();
    let items = Resource::new(
        move || (skip(), slug_items(), offset.get(), search.get()),
        move |(skip_now, s, off, q): (bool, Slug, usize, Option<String>)| async move {
            if skip_now {
                return Ok(Vec::new());
            }
            let parsed = q
                .as_deref()
                .map(SearchQuery::parse)
                .filter(|sq| !sq.is_empty());
            fetch_collections(s, off, page_size, parsed).await
        },
    );

    let slug_count = slug;
    let count = Resource::new(
        move || (slug_count(), search.get()),
        |(s, q): (Slug, Option<String>)| async move {
            let parsed = q
                .as_deref()
                .map(SearchQuery::parse)
                .filter(|sq| !sq.is_empty());
            fetch_collections_count(s, parsed).await
        },
    );

    (items, count)
}
