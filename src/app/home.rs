use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};

use crate::app::{
    collections::{fetch_collections, fetch_collections_count},
    common::CardsLoading,
    icons::{AudioIcon, MovieIcon},
    model::{Collection, MediaKind, Section},
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
        view! {
            <div class="min-h-screen bg-[#0c0b1a] text-white">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4 md:py-6 flex flex-col">
                    <SearchBar search_query=this.search_query offset_reset=|| {} />
                    <ResourceView resource=this.sections view_fn=AllSections
                        adapter=move |s| AllSectionsProps {
                            sections: s,
                            search_query: this.search_query,
                        }/>
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
    let slug = section.slug.clone();
    let slug_a = slug.clone();
    let slug_b = slug.clone();
    let href = section.href();
    let title = section.title.clone();
    let icon = match section.media_kind {
        MediaKind::Video => Either::Left(MovieIcon()),
        MediaKind::Audio => Either::Right(AudioIcon()),
    };

    let offset = RwSignal::new(0usize);
    let collections = Resource::new(
        move || (slug_a.clone(), offset.get(), search_query.get()),
        |(s, off, q)| fetch_collections(s, off, MEDIA_LIST_SIZE, q),
    );
    let _count = Resource::new(
        move || (slug_b.clone(), search_query.get()),
        |(s, q)| fetch_collections_count(s, q),
    );

    let adapter = |c: Vec<Collection>| CollectionStripProps { collections: c };

    view! {
        <section class="bg-white/5 rounded-2xl p-4 md:p-6 my-6">
            <div class="flex items-center justify-between mb-6">
                <a href=href class="flex items-center gap-3 hover:opacity-80">
                    {icon}
                    <span class="text-lg font-bold text-white">{title}</span>
                </a>
            </div>
            <ResourceView resource=collections view_fn=CollectionStrip
                adapter=adapter
                fallback=CardsLoading/>
        </section>
    }
}

#[component]
fn CollectionStrip(collections: Vec<Collection>) -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-4 md:gap-6">
            <For each=move || collections.clone() key=|c| c.id let:c>
                <a href=format!("/s/{}/{}", c.section_slug, c.id)
                    class="group rounded-2xl overflow-hidden bg-[#1a1a24]/80 border border-white/5 hover:border-cyan-400/40 transition">
                    <div class="aspect-[2/3] bg-white/5 overflow-hidden">
                        {c.poster.clone().map(|p| view! {
                            <img src=p class="w-full h-full object-cover group-hover:scale-105 transition"/>
                        })}
                    </div>
                    <div class="p-3">
                        <div class="text-sm text-white truncate">{c.title.clone()}</div>
                    </div>
                </a>
            </For>
        </div>
    }
}
