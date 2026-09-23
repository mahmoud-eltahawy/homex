use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};

use crate::app::{
    common::{
        CardsLoading, CollectionCount, CollectionGrid, CollectionPage, EmptyState,
        collections_paginated, refetch_on_success,
    },
    icons::{DeleteIcon, ViewAllIcon, icon_for},
    inline_edit::{EditableText, use_edit_mode},
    model::{Collection, MediaKind, Section},
    pagination::{PaginationControls, PaginationControlsProps},
    resource_view::ResourceView,
    search::SearchBar,
    sections::{create_section, delete_section, fetch_sections, patch_section_title},
};

const MEDIA_LIST_SIZE: usize = 6;

// ─── Page ─────────────────────────────────────────────────────────────────

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

        let adapter = move |value: Vec<Section>| AllSectionsProps {
            sections: value,
            search_query,
            sections_resource: sections,
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

// ─── Section list ─────────────────────────────────────────────────────────

#[component]
fn AllSections(
    sections: Vec<Section>,
    search_query: RwSignal<Option<String>>,
    sections_resource: Resource<Result<Vec<Section>, ServerFnError>>,
) -> impl IntoView {
    let list = sections
        .into_iter()
        .map(|section| {
            view! {
                <SectionTeaser
                    section=section
                    search_query=search_query
                    sections_resource=sections_resource
                />
            }
        })
        .collect_view();

    let edit_on = use_edit_mode();

    view! {
        {list}
        <Show when=move || edit_on.get()>
            <NewSectionForm sections_resource=sections_resource/>
        </Show>
    }
}

// ─── Section teaser ───────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct SectionTeaserState {
    folded: RwSignal<bool>,
    offset: RwSignal<usize>,
    items: CollectionPage,
    count: CollectionCount,
}

/// All state for one section teaser: fold toggle, pagination offset,
/// items list, and item count. Resets `offset` when the search query changes.
fn use_section_teaser_state(
    slug: String,
    search_query: RwSignal<Option<String>>,
) -> SectionTeaserState {
    let folded = RwSignal::new(false);
    let offset = RwSignal::new(0usize);

    Effect::new(move |_| {
        let _ = search_query.get();
        offset.set(0);
    });

    let slug_for_fetch = slug;
    let (items, count) = collections_paginated(
        move || slug_for_fetch.clone(),
        offset,
        search_query,
        move || folded.get(),
        MEDIA_LIST_SIZE,
    );

    SectionTeaserState {
        folded,
        offset,
        items,
        count,
    }
}

#[component]
fn SectionTeaser(
    section: Section,
    search_query: RwSignal<Option<String>>,
    sections_resource: Resource<Result<Vec<Section>, ServerFnError>>,
) -> impl IntoView {
    let state = use_section_teaser_state(section.slug.clone(), search_query);

    let order = move || if state.folded.get() { "1" } else { "0" };

    let section_for_header = section.clone();
    let header_adapter = move |c: usize| SectionHeaderProps {
        section: section_for_header.clone(),
        count: c,
        folded: state.folded,
        sections_resource,
    };
    let pagination_adapter = move |c: usize| PaginationControlsProps {
        offset: state.offset,
        count: c,
        window_size: 5,
        page_size: MEDIA_LIST_SIZE,
    };
    let content_adapter = move |list: Vec<Collection>| CollectionStripProps { collections: list };

    view! {
        <div style:order=order>
            <hr class="border-t border-white/5 my-10 md:my-12" />
            <section class="bg-white/5 rounded-2xl p-4 md:p-6">
                <ResourceView resource=state.count view_fn=SectionHeader adapter=header_adapter/>
                <Show when=move || !state.folded.get()>
                    <ResourceView
                        resource=state.items
                        view_fn=CollectionStrip
                        adapter=content_adapter
                        fallback=CardsLoading
                    />
                    <ResourceView
                        resource=state.count
                        view_fn=PaginationControls
                        adapter=pagination_adapter
                    />
                </Show>
            </section>
        </div>
    }
}

// ─── Section header ───────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct SectionHeaderState {
    title: RwSignal<String>,
    commit_title: Callback<String>,
    delete_action: Action<u64, Result<(), ServerFnError>>,
}

/// Local title signal + the two server actions. Wires both actions to
/// refetch the sections list on success.
fn use_section_header_state(
    section: &Section,
    sections_resource: Resource<Result<Vec<Section>, ServerFnError>>,
) -> SectionHeaderState {
    let id = section.id;
    let title = RwSignal::new(section.title.clone());

    let rename = Action::new_local(|(id, t): &(u64, String)| patch_section_title(*id, t.clone()));
    let delete_action = Action::new_local(|id: &u64| delete_section(*id));

    refetch_on_success(rename, sections_resource);
    refetch_on_success(delete_action, sections_resource);

    let commit_title = Callback::new(move |v: String| {
        title.set(v.clone());
        let _ = rename.dispatch((id, v));
    });

    SectionHeaderState {
        title,
        commit_title,
        delete_action,
    }
}

#[component]
fn SectionHeader(
    section: Section,
    count: usize,
    folded: RwSignal<bool>,
    sections_resource: Resource<Result<Vec<Section>, ServerFnError>>,
) -> impl IntoView {
    let state = use_section_header_state(&section, sections_resource);
    let edit_on = use_edit_mode();

    let id = section.id;
    let kind = section.media_kind();
    let href_display = section.href();
    let href_actions = section.href();

    let on_delete = Callback::new(move |_| {
        let _ = state.delete_action.dispatch(id);
    });
    let nested = section.nested;

    view! {
        <div class="flex items-center justify-between mb-6 gap-3">
            <Show
                when=move || edit_on.get()
                fallback=move || view! {
                    <SectionHeaderViewMode
                        href=href_display.clone()
                        title=state.title
                        count
                        kind
                        nested
                    />
                }
            >
                <SectionHeaderEditMode
                    title=state.title
                    on_commit=state.commit_title
                    count
                    kind
                    nested
                />
            </Show>
            <SectionHeaderActions
                view_all_href=href_actions
                edit_on=edit_on
                on_delete=on_delete
                folded=folded
            />
        </div>
    }
}

#[component]
fn SectionHeaderViewMode(
    #[prop(into)] href: String,
    title: RwSignal<String>,
    count: usize,
    kind: MediaKind,
    nested: bool,
) -> impl IntoView {
    view! {
        <a
            href=href
            class="flex items-center gap-3 group min-w-0 flex-1"
        >
            <span class="flex items-center shrink-0">{icon_for(kind,nested)}</span>
            <span class="text-lg font-bold text-white group-hover:text-cyan-300 transition truncate">
                {move || title.get()}
            </span>
            <span class="text-sm font-mono text-white/60 bg-white/10 px-3 py-0.5 rounded-full shrink-0">
                {count}
            </span>
        </a>
    }
}

#[component]
fn SectionHeaderEditMode(
    title: RwSignal<String>,
    count: usize,
    kind: MediaKind,
    on_commit: Callback<String>,
    nested: bool,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-3 min-w-0 flex-1">
            <span class="flex items-center shrink-0">{icon_for(kind,nested)}</span>
            <EditableText
                value=Signal::derive(move || title.get())
                on_commit=on_commit
                class="text-lg font-bold text-white truncate"
            />
            <span class="text-sm font-mono text-white/60 bg-white/10 px-3 py-0.5 rounded-full shrink-0">
                {count}
            </span>
        </div>
    }
}

#[component]
fn SectionHeaderActions(
    #[prop(into)] view_all_href: String,
    edit_on: RwSignal<bool>,
    on_delete: Callback<()>,
    folded: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-1 shrink-0">
            <Show when=move || edit_on.get()>
                <button
                    type="button"
                    on:click=move |_| on_delete.run(())
                    class="p-1 rounded hover:bg-red-500/20 text-red-300 transition-colors"
                    aria-label="Delete section"
                >
                    <DeleteIcon/>
                </button>
            </Show>
            <FoldButton folded/>
            <a
                href=view_all_href
                class="p-1 rounded hover:bg-white/10 transition-colors"
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
        return Either::Left(view! { <EmptyState label="0"/> });
    }
    Either::Right(view! { <CollectionGrid collections=collections/> })
}

// ─── New-section form ─────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct NewSectionFormState {
    title: RwSignal<String>,
    kind: RwSignal<String>,
    nested: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    create: Action<(String, String, bool), Result<u64, ServerFnError>>,
}

/// The form's signals, the create action, and the effect that resets
/// the form on success or surfaces the server error on failure.
fn use_new_section_form(
    sections_resource: Resource<Result<Vec<Section>, ServerFnError>>,
) -> NewSectionFormState {
    let title = RwSignal::new(String::new());
    let kind = RwSignal::new("video".to_string());
    let nested = RwSignal::new(false);
    let error = RwSignal::new(None::<String>);

    let create = Action::new_local(|(t, k, n): &(String, String, bool)| {
        create_section(t.clone(), k.clone(), *n)
    });

    refetch_on_success(create, sections_resource);

    Effect::new(move |_| match create.value().get() {
        Some(Ok(_)) => {
            title.set(String::new());
            error.set(None);
        }
        Some(Err(e)) => error.set(Some(e.to_string())),
        None => {}
    });

    NewSectionFormState {
        title,
        kind,
        nested,
        error,
        create,
    }
}

#[component]
fn NewSectionForm(
    sections_resource: Resource<Result<Vec<Section>, ServerFnError>>,
) -> impl IntoView {
    let state = use_new_section_form(sections_resource);

    let submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let t = state.title.get_untracked().trim().to_string();
        if t.is_empty() {
            state.error.set(Some("Name is required".into()));
            return;
        }
        state.error.set(None);
        state
            .create
            .dispatch((t, state.kind.get_untracked(), state.nested.get_untracked()));
    };

    view! {
        <div class="mt-12 border-2 border-dashed border-cyan-400/30 rounded-2xl p-6 bg-cyan-500/[0.03]">
            <h3 class="text-lg font-bold text-white mb-4">"Add new section"</h3>
            <form on:submit=submit class="flex flex-wrap gap-3 items-end">
                <label class="flex flex-col text-sm flex-1 min-w-48">
                    <span class="mb-1 text-gray-300">"Name"</span>
                    <input
                        type="text"
                        prop:value=move || state.title.get()
                        on:input=move |e| state.title.set(event_target_value(&e))
                        placeholder="Movies, series, albums..."
                        class="bg-white/10 rounded-lg px-3 py-1.5 text-white w-full"
                    />
                </label>
                <label class="flex flex-col text-sm">
                    <span class="mb-1 text-gray-300">"Type"</span>
                    <select
                        prop:value=move || state.kind.get()
                        on:change=move |e| state.kind.set(event_target_value(&e))
                        class="bg-white/10 rounded-lg px-3 py-1.5 text-white"
                    >
                        <option value="video">"Video"</option>
                        <option value="audio">"Audio"</option>
                    </select>
                </label>
                <label class="flex items-center gap-2 text-sm pb-2 text-gray-300">
                    <input
                        type="checkbox"
                        prop:checked=move || state.nested.get()
                        on:change=move |e| state.nested.set(event_target_checked(&e))
                    />
                    <span>"Groups"</span>
                </label>
                <button
                    type="submit"
                    disabled=move || state.create.pending().get()
                    class="px-4 py-2 rounded-xl bg-gradient-to-r from-cyan-500 to-blue-500 \
                           hover:from-cyan-400 hover:to-blue-400 text-white font-bold text-sm \
                           disabled:opacity-50"
                >
                    {move || if state.create.pending().get() {
                        "Adding..."
                    } else {
                        "Add"
                    }}
                </button>
            </form>
            {move || state.error.get().map(|e| view! {
                <div class="mt-3 text-red-300 text-sm">{e}</div>
            })}
        </div>
    }
}
