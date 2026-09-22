use crate::app::{
    collections::create_empty_collection,
    common::{
        CardsLoading, CollectionCount, CollectionGrid, CollectionGridProps, CollectionPage,
        collections_paginated,
    },
    inline_edit::use_edit_mode,
    model::{Collection, Section},
    pagination::{PaginationControls, PaginationControlsProps},
    resource_view::ResourceView,
    route_params::use_string_param,
    search::SearchBar,
    sections::fetch_section_by_slug,
};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::{LazyRoute, lazy_route};
use server_fn::ServerFnError;

const LISTING_PAGE_SIZE: usize = 18;

pub struct SectionListingPage {
    section: Resource<Result<Section, ServerFnError>>,
    collections: CollectionPage,
    count: CollectionCount,
    offset: RwSignal<usize>,
    search_query: RwSignal<Option<String>>,
}

#[lazy_route]
impl LazyRoute for SectionListingPage {
    fn data() -> Self {
        let slug = use_string_param("slug");
        let offset = RwSignal::new(0);
        let search_query = RwSignal::new(None);

        let slug_for_section = slug;
        let section = Resource::new(slug_for_section, fetch_section_by_slug);

        let slug_for_fetch = slug;
        let (collections, count) = collections_paginated(
            slug_for_fetch,
            offset,
            search_query,
            || false,
            LISTING_PAGE_SIZE,
        );

        Self {
            section,
            collections,
            count,
            offset,
            search_query,
        }
    }

    fn view(this: Self) -> AnyView {
        let offset = this.offset;
        let search_query = this.search_query;
        let collections = this.collections;
        let count = this.count;
        let adapter = move |section: Section| SectionListingBodyProps {
            section,
            collections,
            count,
            offset,
            search_query,
        };
        view! {
            <ResourceView resource=this.section view_fn=SectionListingBody adapter=adapter/>
        }
        .into_any()
    }
}

#[component]
fn SectionListingBody(
    section: Section,
    collections: Resource<Result<Vec<Collection>, ServerFnError>>,
    count: Resource<Result<usize, ServerFnError>>,
    offset: RwSignal<usize>,
    search_query: RwSignal<Option<String>>,
) -> impl IntoView {
    let slug = section.slug.clone();
    let new_label = section.new_label().to_string();

    let pag_adapter = move |c: usize| PaginationControlsProps {
        offset,
        count: c,
        window_size: 8,
        page_size: LISTING_PAGE_SIZE,
    };
    let grid_adapter = move |list: Vec<Collection>| CollectionGridProps { collections: list };

    view! {
        <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
            <div class="flex justify-end mb-3">
                <CreateNewButton section_slug=slug label=new_label/>
            </div>
            <SearchBar search_query offset_reset=move || offset.set(0)/>
            <ResourceView
                resource=collections
                view_fn=CollectionGrid
                adapter=grid_adapter
                fallback=CardsLoading
            />
            <ResourceView resource=count view_fn=PaginationControls adapter=pag_adapter/>
        </div>
    }
}

// ─── Create-new-collection flow ───────────────────────────────────────────

/// Dispatches `create_empty_collection` and navigates to the new collection
/// on success. Returns the action so the caller can wire the button.
fn use_create_collection_nav(section_slug: String) -> Action<String, Result<u64, ServerFnError>> {
    let action = Action::new_local(move |s: &String| create_empty_collection(s.clone()));
    let navigate = use_navigate();
    let slug_for_nav = section_slug;

    Effect::new(move |_| {
        if let Some(Ok(id)) = action.value().get() {
            let href = format!("/s/{}/{}", slug_for_nav, id);
            navigate(&href, Default::default());
        }
    });

    action
}

#[component]
fn CreateNewButton(
    #[prop(into)] section_slug: String,
    #[prop(into)] label: String,
) -> impl IntoView {
    let action = use_create_collection_nav(section_slug.clone());
    let slug_for_action = section_slug;
    let label_for_view = label;
    let edit_on = use_edit_mode();

    view! {
        <Show when=move || edit_on.get()>
            <button
                type="button"
                on:click={
                    let slug = slug_for_action.clone();
                    move |_| { let _ = action.dispatch(slug.clone()); }
                }
                disabled=move || action.pending().get()
                class="inline-flex items-center gap-2 px-4 py-2 rounded-xl \
                       bg-gradient-to-r from-cyan-500 to-blue-500 \
                       hover:from-cyan-400 hover:to-blue-400 text-white font-bold text-sm \
                       shadow-lg shadow-cyan-500/20 transition-all hover:scale-105 \
                       disabled:opacity-50"
            >
                <span class="text-lg leading-none">"+"</span>
                {
                    let label_for_view = label_for_view.clone();
                    move || if action.pending().get() {
                        "جاري الإنشاء...".to_string()
                    } else {
                        label_for_view.clone()
                    }
                }
            </button>
        </Show>
    }
}
