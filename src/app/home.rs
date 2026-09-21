use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};

use crate::app::{
    common::{CardsLoading, CollectionGrid, EmptyState, collections_paginated, refetch_on_success},
    icons::{DeleteIcon, ViewAllIcon, icon_for},
    inline_edit::{EditModeToggle, EditableText, provide_edit_mode, use_edit_mode},
    model::{Collection, Section},
    pagination::{PaginationControls, PaginationControlsProps},
    resource_view::ResourceView,
    search::SearchBar,
    sections::{create_section, delete_section, fetch_sections, patch_section_title},
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
        let edit_on = provide_edit_mode();

        let adapter = move |value: Vec<Section>| AllSectionsProps {
            sections: value,
            search_query,
            sections_resource: sections,
        };

        view! {
            <div class="min-h-screen bg-[#0c0b1a] text-white">
                <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-4 md:py-6 flex flex-col">
                    <div class="flex justify-end mb-2">
                        <EditModeToggle edit_on=edit_on/>
                    </div>
                    <SearchBar search_query=search_query offset_reset=|| {} />
                    <ResourceView resource=sections view_fn=AllSections adapter=adapter/>
                </div>
            </div>
        }
        .into_any()
    }
}

// ─── Section list ────────────────────────────────────────────────────────

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

#[component]
fn SectionTeaser(
    section: Section,
    search_query: RwSignal<Option<String>>,
    sections_resource: Resource<Result<Vec<Section>, ServerFnError>>,
) -> impl IntoView {
    let folded = RwSignal::new(false);
    let offset = RwSignal::new(0usize);

    let slug = section.slug.clone();
    let slug_for_fetch = slug.clone();

    Effect::new(move |_| {
        let _ = search_query.get();
        offset.set(0);
    });

    let (items, count) = collections_paginated(
        {
            let slug_for_fetch = slug_for_fetch.clone();
            move || slug_for_fetch.clone()
        },
        offset,
        search_query,
        move || folded.get(),
        MEDIA_LIST_SIZE,
    );

    let order = move || if folded.get() { "1" } else { "0" };

    let section_for_header = section.clone();
    let header_adapter = move |c: usize| SectionHeaderProps {
        section: section_for_header.clone(),
        count: c,
        folded,
        sections_resource,
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
                <ResourceView resource=count view_fn=SectionHeader adapter=header_adapter/>
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

// ─── Section header (inline editable) ────────────────────────────────────

#[component]
fn SectionHeader(
    section: Section,
    count: usize,
    folded: RwSignal<bool>,
    sections_resource: Resource<Result<Vec<Section>, ServerFnError>>,
) -> impl IntoView {
    let edit_on = use_edit_mode();
    let id = section.id;
    let kind = section.media_kind;
    let href_display = section.href();
    let href_actions = section.href();
    let title = RwSignal::new(section.title.clone());

    let rename = Action::new_local(|(id, t): &(u64, String)| patch_section_title(*id, t.clone()));
    let delete_action = Action::new_local(|id: &u64| delete_section(*id));

    refetch_on_success(rename, sections_resource);
    refetch_on_success(delete_action, sections_resource);

    let commit_title = Callback::new(move |v: String| {
        title.set(v.clone());
        rename.dispatch((id, v));
    });

    view! {
        <div class="flex items-center justify-between mb-6 gap-3">
            <Show
                when=move || edit_on.get()
                fallback=move || view! {
                    <a
                        href=href_display.clone()
                        class="flex items-center gap-3 group min-w-0 flex-1"
                    >
                        <span class="flex items-center shrink-0">{icon_for(kind)}</span>
                        <span class="text-lg font-bold text-white group-hover:text-cyan-300 transition truncate">
                            {move || title.get()}
                        </span>
                        <span class="text-sm font-mono text-white/60 bg-white/10 px-3 py-0.5 rounded-full shrink-0">
                            {count}
                        </span>
                    </a>
                }
            >
                <div class="flex items-center gap-3 min-w-0 flex-1">
                    <span class="flex items-center shrink-0">{icon_for(kind)}</span>
                    <EditableText
                        value=Signal::derive(move || title.get())
                        on_commit=commit_title
                        class="text-lg font-bold text-white truncate"
                    />
                    <span class="text-sm font-mono text-white/60 bg-white/10 px-3 py-0.5 rounded-full shrink-0">
                        {count}
                    </span>
                </div>
            </Show>
            <div class="flex items-center gap-1 shrink-0">
                <Show when=move || edit_on.get()>
                    <button
                        type="button"
                        on:click=move |_| {delete_action.dispatch(id);}
                        class="p-1 rounded hover:bg-red-500/20 text-red-300 transition-colors"
                        aria-label="حذف القسم"
                    >
                        <DeleteIcon/>
                    </button>
                </Show>
                <FoldButton folded/>
                <a
                    href=href_actions
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
        return Either::Left(view! { <EmptyState label="0"/> });
    }
    Either::Right(view! { <CollectionGrid collections=collections/> })
}

// ─── New-section form (only visible in edit mode) ────────────────────────

#[component]
fn NewSectionForm(
    sections_resource: Resource<Result<Vec<Section>, ServerFnError>>,
) -> impl IntoView {
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

    let submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let t = title.get_untracked().trim().to_string();
        if t.is_empty() {
            error.set(Some("الاسم مطلوب".into()));
            return;
        }
        error.set(None);
        create.dispatch((t, kind.get_untracked(), nested.get_untracked()));
    };

    view! {
        <div class="mt-12 border-2 border-dashed border-cyan-400/30 rounded-2xl p-6 bg-cyan-500/[0.03]">
            <h3 class="text-lg font-bold text-white mb-4">"إضافة قسم جديد"</h3>
            <form on:submit=submit class="flex flex-wrap gap-3 items-end">
                <label class="flex flex-col text-sm flex-1 min-w-48">
                    <span class="mb-1 text-gray-300">"الاسم"</span>
                    <input
                        type="text"
                        prop:value=move || title.get()
                        on:input=move |e| title.set(event_target_value(&e))
                        placeholder="أفلام، مسلسلات، ألبومات..."
                        class="bg-white/10 rounded-lg px-3 py-1.5 text-white w-full"
                    />
                </label>
                <label class="flex flex-col text-sm">
                    <span class="mb-1 text-gray-300">"النوع"</span>
                    <select
                        prop:value=move || kind.get()
                        on:change=move |e| kind.set(event_target_value(&e))
                        class="bg-white/10 rounded-lg px-3 py-1.5 text-white"
                    >
                        <option value="video">"فيديو"</option>
                        <option value="audio">"صوت"</option>
                    </select>
                </label>
                <label class="flex items-center gap-2 text-sm pb-2 text-gray-300">
                    <input
                        type="checkbox"
                        prop:checked=move || nested.get()
                        on:change=move |e| nested.set(event_target_checked(&e))
                    />
                    <span>"مجموعات"</span>
                </label>
                <button
                    type="submit"
                    disabled=move || create.pending().get()
                    class="px-4 py-2 rounded-xl bg-gradient-to-r from-cyan-500 to-blue-500 \
                           hover:from-cyan-400 hover:to-blue-400 text-white font-bold text-sm \
                           disabled:opacity-50"
                >
                    {move || if create.pending().get() { "جاري الإضافة..." } else { "إضافة" }}
                </button>
            </form>
            {move || error.get().map(|e| view! {
                <div class="mt-3 text-red-300 text-sm">{e}</div>
            })}
        </div>
    }
}
