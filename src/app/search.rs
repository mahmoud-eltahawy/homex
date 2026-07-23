use super::fetch_all_media;
use crate::app::{CardsLoading, MediaCard};
use leptos::{either::Either, prelude::*};
use leptos_router::hooks::use_query_map;

#[component]
pub fn Search() -> impl IntoView {
    let query_map = use_query_map();
    let query = move || {
        query_map.with(|m| {
            m.get("q")
                .map(|s| s.to_string())
                .unwrap_or_default()
                .trim()
                .to_lowercase()
        })
    };
    let all_media = Resource::new(|| (), |_| async move { fetch_all_media().await });
    let results = Memo::new(move |_| {
        let q = query();
        if q.is_empty() {
            return vec![];
        }
        all_media
            .get()
            .and_then(|x| x.ok())
            .map(|media| {
                media
                    .into_iter()
                    .filter(|item| item.title().to_lowercase().contains(&q))
                    .collect()
            })
            .unwrap_or_default()
    });
    view! {
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <SearchHeader/>
            <Suspense fallback=CardsLoading>
                {move || if results.get().is_empty() {
                    Either::Left(NoSearchResults())
                } else {
                    Either::Right(view! {
                        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6">
                            <For each={move || results.get()} key=|m| m.id() let:item>
                                <MediaCard item=item.clone()/>
                            </For>
                        </div>
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn SearchHeaderResults() -> impl IntoView {
    let query_map = use_query_map();
    let q = query_map.with(|m| m.get("q").map(|s| s.to_string()).unwrap_or_default());
    view! {
        <p class="text-gray-400 text-sm sm:text-base">
            "نتائج البحث عن" <span class="text-white font-semibold">{format!("\"{}\"", q)}</span>
        </p>
    }
}

#[component]
fn SearchHeader() -> impl IntoView {
    let query_map = use_query_map();
    let query = move || {
        query_map.with(|m| {
            m.get("q")
                .map(|s| s.to_string())
                .unwrap_or_default()
                .trim()
                .to_lowercase()
        })
    };
    view! {
        <div class="mb-6 md:mb-8">
            <h1 class="text-3xl sm:text-4xl font-black text-white mb-1">"نتائج البحث"</h1>
            {move || if query().is_empty() {
                Either::Left(view! { <p class="text-gray-400 text-sm sm:text-base">"أدخل كلمة بحث للعثور على الوسائط."</p> })
            } else {
                Either::Right(SearchHeaderResults())
            }}
        </div>
    }
}

#[component]
fn NoSearchResults() -> impl IntoView {
    view! { <div class="text-center py-16 text-gray-400 text-sm sm:text-base">"لا يوجد وسائط تطابق بحثك."</div> }
}
