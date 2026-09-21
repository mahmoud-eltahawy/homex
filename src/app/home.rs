use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};

use crate::app::{
    collections::{fetch_collections, fetch_collections_count},
    common::CardsLoading,
    icons::{AudioIcon, EmptyStateIcon, MovieIcon, ViewAllIcon},
    model::{Collection, MediaKind, Section},
    pagination::{PaginationControls, PaginationControlsProps},
    resource_view::ResourceView,
    search::SearchBar,
    sections::fetch_sections,
};

const MEDIA_LIST_SIZE: usize = 6;

pub struct HomePage {
    search_query: RwSignal<Option<String>>,
    sections: Resource<Result<Vec<Section>, ServerFnError>>,
}

#[lazy_route]
impl LazyRoute for HomePage {
    fn data() -> Self {
        Self {
            search_query: RwSignal::new(None),
            sections: Resource::new(|| (), |_| fetch_sections()),
        }
    }

    fn view(this: Self) -> AnyView {
        let search_query = this.search_query;
        let sections = this.sections;
        let adapter = move |s: Vec<Section>| AllSectionsProps {
            sections: s,
            search_query,
        };
        view! {
            <div class="min-h-screen bg-[#0c0b1a] text-white">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4 md:py-6 flex flex-col">
                    <SearchBar search_query=search_query offset_reset=|| {} />
                    <ResourceView resource=sections view_fn=AllSections adapter=adapter/>
                </div>
            </div>
        }
        .into_any()
    }
}

#[component]
fn AllSections(sections: Vec<Section>, search_query: RwSignal<Option<String>>) -> impl IntoView {
    view! {
        <For each=move || sections.clone() key=|s| s.id let:section>
            <SectionTeaser section=section search_query=search_query/>
        </For>
    }
}

#[component]
fn SectionTeaser(section: Section, search_query: RwSignal<Option<String>>) -> impl IntoView {
    let folded = RwSignal::new(false);
    let offset = RwSignal::new(0usize);

    let kind = section.media_kind;
    let slug_for_items = section.slug.clone();
    let slug_for_count = section.slug.clone();
    let href = section.href();
    let title = section.title.clone();

    // Any new search resets every section's page back to 1.
    Effect::new(move |_| {
        let _ = search_query.get();
        offset.set(0);
    });

    let items = Resource::new(
        move || {
            (
                folded.get(),
                slug_for_items.clone(),
                offset.get(),
                search_query.get(),
            )
        },
        move |(is_folded, s, off, q)| async move {
            if is_folded {
                Ok(Vec::new())
            } else {
                fetch_collections(s, off, MEDIA_LIST_SIZE, q).await
            }
        },
    );
    let count = Resource::new(
        move || (slug_for_count.clone(), search_query.get()),
        |(s, q)| fetch_collections_count(s, q),
    );

    // Folded sections slide to the bottom of the flex column.
    let order = move || if folded.get() { "1" } else { "0" };

    let header_adapter = move |c: usize| SectionHeaderProps {
        kind,
        title: title.clone(),
        count: c,
        href: href.clone(),
        folded,
    };

    let pagination_adapter = move |c: usize| PaginationControlsProps {
        offset,
        count: c,
        window_size: 5,
        page_size: MEDIA_LIST_SIZE,
    };

    let content_adapter = move |list: Vec<Collection>| CollectionStripProps { collections: list };

    view! {
        <div style:order=order>
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
                        view_fn=CollectionStrip
                        adapter=content_adapter
                        fallback=CardsLoading
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
    kind: MediaKind,
    title: String,
    count: usize,
    href: String,
    folded: RwSignal<bool>,
) -> impl IntoView {
    let icon = match kind {
        MediaKind::Video => Either::Left(MovieIcon()),
        MediaKind::Audio => Either::Right(AudioIcon()),
    };
    view! {
        <div class="flex items-center justify-between mb-6">
            <a href=href.clone() class="flex items-center gap-3 group min-w-0">
                <span class="flex items-center shrink-0">{icon}</span>
                <span class="text-lg font-bold text-white group-hover:text-cyan-300 transition truncate">
                    {title}
                </span>
                <span class="text-sm font-mono text-white/60 bg-white/10 px-3 py-0.5 rounded-full shrink-0">
                    {count}
                </span>
            </a>
            <div class="flex items-center gap-1 shrink-0">
                <FoldButton folded/>
                <a
                    href=href
                    class="p-1 rounded hover:bg-white/10 transition-colors"
                    aria-label="View all"
                >
                    <ViewAllIcon />
                </a>
            </div>
        </div>
    }
}

#[component]
fn FoldButton(folded: RwSignal<bool>) -> impl IntoView {
    let on_click = move |_| folded.update(|x| *x = !*x);
    view! {
        <button
            type="button"
            on:click=on_click
            class="p-1 rounded hover:bg-white/10 transition-colors"
            aria-label="Toggle section"
        >
            <svg
                class="w-5 h-5 transition-transform duration-200"
                style=move || format!(
                    "transform: rotate({}deg)",
                    if folded.get() { 180 } else { 0 },
                )
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
fn CollectionStrip(collections: Vec<Collection>) -> impl IntoView {
    if collections.is_empty() {
        return Either::Left(view! {
            <div class="flex flex-col items-center justify-center py-12 gap-3">
                <EmptyStateIcon />
                <span class="text-white/20 text-sm font-mono">"0"</span>
            </div>
        });
    }
    Either::Right(view! {
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6">
            <For each=move || collections.clone() key=|c| c.id let:c>
                <CollectionCard collection=c/>
            </For>
        </div>
    })
}

#[component]
pub fn CollectionCard(collection: Collection) -> impl IntoView {
    let href = format!("/s/{}/{}", collection.section_slug, collection.id);
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
