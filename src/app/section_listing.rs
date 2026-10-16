use crate::app::{
    common::{
        CardsLoading, CollectionCount, CollectionGrid, CollectionGridProps, CollectionPage,
        collections_paginated,
    },
    error::AppError,
    ids::Slug,
    inline_edit::use_edit_mode,
    model::{Collection, Section},
    pagination::{PaginationControls, PaginationControlsProps},
    route_params::use_slug_param,
    search::SearchBar,
    sections::fetch_section_by_slug,
};
use leptos::prelude::*;
use leptos_outcome::ResourceView;
use leptos_page::{Page, PageContent, RouteError};
use leptos_router::hooks::use_navigate;

const LISTING_PAGE_SIZE: usize = 18;

pub struct SectionListingContent {
    section: Resource<Result<Section, AppError>>,
    collections: CollectionPage,
    count: CollectionCount,
    offset: RwSignal<usize>,
    search_query: RwSignal<Option<String>>,
}

impl PageContent<RouteError> for SectionListingContent {
    fn load() -> Result<Self, RouteError> {
        let slug = use_slug_param("slug")?;

        let offset = RwSignal::new(0);
        let search_query = RwSignal::new(None);

        let slug_for_section = slug.clone();
        let section = Resource::new(
            move || slug_for_section.clone(),
            |s: Slug| async move { fetch_section_by_slug(s).await },
        );

        let slug_for_fetch = slug;
        let (collections, count) = collections_paginated(
            move || slug_for_fetch.clone(),
            offset,
            search_query,
            || false,
            LISTING_PAGE_SIZE,
        );

        Ok(Self {
            section,
            collections,
            count,
            offset,
            search_query,
        })
    }

    fn render(self) -> impl IntoView {
        let Self {
            section,
            collections,
            count,
            offset,
            search_query,
        } = self;

        let adapter = move |section: Section| SectionListingBodyProps {
            section,
            collections,
            count,
            offset,
            search_query,
        };
        view! {
            <ResourceView resource=section view_fn=SectionListingBody adapter=adapter/>
        }
    }
}

pub type SectionListingPage = Page<SectionListingContent, RouteError>;

#[component]
fn SectionListingBody(
    section: Section,
    collections: Resource<Result<Vec<Collection>, AppError>>,
    count: Resource<Result<usize, AppError>>,
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

#[component]
fn CreateNewButton(#[prop(into)] section_slug: Slug, #[prop(into)] label: String) -> impl IntoView {
    let navigate = use_navigate();
    let edit_on = use_edit_mode();

    let slug = section_slug;
    let on_click = move |_| {
        let href = format!("/new-collection/{}", slug);
        navigate(&href, Default::default());
    };

    view! {
        <Show when=move || edit_on.get()>
            <button
                type="button"
                on:click=on_click.clone()
                class="inline-flex items-center gap-2 px-4 py-2 rounded-xl \
                       bg-gradient-to-r from-cyan-500 to-blue-500 \
                       hover:from-cyan-400 hover:to-blue-400 text-white font-bold text-sm \
                       shadow-lg shadow-cyan-500/20 transition-all hover:scale-105"
            >
                <span class="text-lg leading-none">"+"</span>
                {label.clone()}
            </button>
        </Show>
    }
}
