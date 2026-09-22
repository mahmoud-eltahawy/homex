use crate::app::collections::{fetch_collections, fetch_collections_count};
use crate::app::icons::EmptyStateIcon;
use crate::app::model::Collection;
use leptos::prelude::*;
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
            <div class="p-3">
                <div class="text-sm text-white font-semibold truncate">{title}</div>
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

pub fn refetch_on_success<V, A, R>(
    action: Action<A, Result<V, ServerFnError>>,
    resource: Resource<R>,
) where
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

pub type CollectionPage = Resource<Result<Vec<Collection>, ServerFnError>>;
pub type CollectionCount = Resource<Result<usize, ServerFnError>>;

pub fn collections_paginated(
    slug: impl Fn() -> String + Clone + Send + Sync + 'static,
    offset: RwSignal<usize>,
    search: RwSignal<Option<String>>,
    skip: impl Fn() -> bool + Copy + Send + Sync + 'static,
    page_size: usize,
) -> (CollectionPage, CollectionCount) {
    let slug_items = slug.clone();
    let items = Resource::new(
        move || (skip(), slug_items(), offset.get(), search.get()),
        move |(skip_now, s, off, q)| async move {
            if skip_now {
                Ok(Vec::new())
            } else {
                fetch_collections(s, off, page_size, q).await
            }
        },
    );

    let slug_count = slug.clone();
    let count = Resource::new(
        move || (slug_count(), search.get()),
        |(s, q)| fetch_collections_count(s, q),
    );

    (items, count)
}

#[derive(Clone, Copy)]
struct Bundle<T>(T);

pub trait ContextBundle: Clone + Send + Sync + 'static {
    fn provide(self) {
        provide_context(Bundle(self));
    }

    fn expect() -> Self {
        expect_context::<Bundle<Self>>().0
    }
}
