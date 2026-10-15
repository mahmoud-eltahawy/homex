use crate::app::error::AppError;
use crate::app::model::MediaKind;
use crate::app::unclassified::fetch_sections_with_collections;
use crate::app::unclassified::helpers::{file_stem, kind_for_ext, parse_season};
use crate::app::unclassified::jobs::ClassifyJob;
use crate::app::unclassified::tracker::BatchPhase;
use crate::app::unclassified::types::{
    ClassifyDestination, ClassifyInput, SectionWithCollections, UnclassifiedEntry,
};
use leptos::prelude::*;
use leptos_outcome::ErrorView;
use uuid::Uuid;

use super::primitives::{FormActions, ModalShell, SeasonField, SectionSelect, TitleField};
use super::progress::{BatchProgress, PhaseFinal};

#[derive(Clone, Copy)]
struct FileFormState {
    title: RwSignal<String>,
    section_slug: RwSignal<String>,
    collection_id: RwSignal<Option<Uuid>>,
    season: RwSignal<String>,
    creating_new: RwSignal<bool>,
    new_title: RwSignal<String>,
}

impl FileFormState {
    fn new() -> Self {
        Self {
            title: RwSignal::new(String::new()),
            section_slug: RwSignal::new(String::new()),
            collection_id: RwSignal::new(None),
            season: RwSignal::new(String::new()),
            creating_new: RwSignal::new(false),
            new_title: RwSignal::new(String::new()),
        }
    }

    fn reset(&self) {
        self.title.set(String::new());
        self.section_slug.set(String::new());
        self.collection_id.set(None);
        self.season.set(String::new());
        self.creating_new.set(false);
        self.new_title.set(String::new());
    }
}

#[component]
pub fn ClassifyModal(
    selection: RwSignal<Vec<UnclassifiedEntry>>,
    open: RwSignal<bool>,
    job: ClassifyJob,
    on_close: Callback<()>,
) -> impl IntoView {
    let file_kind = Memo::new(move |_| {
        selection
            .get()
            .first()
            .and_then(|e| kind_for_ext(e.ext.as_deref()))
    });

    let sections: Resource<Result<Vec<SectionWithCollections>, AppError>> = Resource::new(
        move || open.get().then(|| file_kind.get()).flatten(),
        |k| async move {
            match k {
                Some(kind) => fetch_sections_with_collections(kind).await,
                None => Ok(Vec::new()),
            }
        },
    );
    let form = FileFormState::new();
    let is_single = Signal::derive(move || selection.get().len() == 1);

    Effect::new(move |_| {
        if !open.get() {
            return;
        }
        let sel = selection.get_untracked();
        form.reset();
        if let [single] = sel.as_slice() {
            form.title.set(file_stem(&single.name));
        }
    });

    Effect::new(move |_| {
        if !open.get() {
            job.reset();
        }
    });

    Effect::new(move |_| {
        let _ = form.section_slug.get();
        form.collection_id.set(None);
        form.creating_new.set(false);
        form.new_title.set(String::new());
    });

    Effect::new(move |_| {
        if form.creating_new.get()
            && form.new_title.get_untracked().is_empty()
            && let Some(first) = selection.get_untracked().first()
        {
            form.new_title.set(file_stem(&first.name));
        }
    });

    let chosen_section: Signal<Option<SectionWithCollections>> = Signal::derive(move || {
        let slug = form.section_slug.get();
        if slug.is_empty() {
            return None;
        }
        sections
            .get()
            .and_then(|r| r.ok())
            .and_then(|list| list.into_iter().find(|s| s.slug == slug))
    });

    let chosen_is_series = Signal::derive(move || {
        file_kind.get() == Some(MediaKind::Video)
            && chosen_section.get().map(|s| s.nested).unwrap_or(false)
    });

    Effect::new(move |_| {
        if chosen_is_series.get() && form.season.get_untracked().trim().is_empty() {
            form.season.set("1".to_string());
        }
    });

    let tracker = job.tracker();
    let phase = tracker.phase();
    let is_busy = Signal::derive(move || matches!(phase.get(), BatchPhase::Running));

    let show_setup = Signal::derive(move || matches!(phase.get(), BatchPhase::Setup));
    let show_running = Signal::derive(move || matches!(phase.get(), BatchPhase::Running));
    let show_final = Signal::derive(move || {
        matches!(
            phase.get(),
            BatchPhase::Done
                | BatchPhase::Cancelled
                | BatchPhase::Partial { .. }
                | BatchPhase::Failed(_)
        )
    });

    let can_move = Signal::derive(move || {
        if form.creating_new.get() {
            !form.section_slug.get().is_empty() && !form.new_title.get().trim().is_empty()
        } else {
            form.collection_id.get().is_some()
        }
    });

    let confirm_label = Signal::derive(move || {
        let n = selection.get().len();
        if n <= 1 {
            "Move".to_string()
        } else {
            format!("Move {n} files")
        }
    });

    let subtitle = Signal::derive(move || {
        let sel = selection.get();
        match sel.len() {
            0 => String::new(),
            1 => sel[0].name.clone(),
            n => format!("{n} files"),
        }
    });

    let success_heading = Signal::derive(move || {
        let n = selection.get().len();
        if n <= 1 {
            "Moved successfully".to_string()
        } else {
            format!("Moved {n} files")
        }
    });
    let success_detail = Signal::derive(|| "The items are now in the collection.".to_string());

    let link: Signal<Option<(String, Option<Uuid>)>> = Signal::derive(move || {
        tracker
            .slots()
            .get()
            .first()
            .map(|s| (s.section_slug.clone(), s.collection_id))
    });

    let cancel_pending = RwSignal::new(false);

    let try_close = Callback::new(move |_: ()| {
        if is_busy.get_untracked() {
            return;
        }
        let refresh = matches!(
            phase.get_untracked(),
            BatchPhase::Done | BatchPhase::Partial { .. }
        );
        open.set(false);
        if refresh {
            on_close.run(());
        }
    });

    let on_cancel = Callback::new(move |_: ()| {
        if cancel_pending.get_untracked() {
            return;
        }
        cancel_pending.set(true);
        job.cancel();
    });

    let confirm = Callback::new(move |_: ()| {
        let sel = selection.get_untracked();
        if sel.is_empty() {
            return;
        }

        let dest = if form.creating_new.get_untracked() {
            let slug = form.section_slug.get_untracked();
            let title = form.new_title.get_untracked();
            if slug.is_empty() || title.trim().is_empty() {
                return;
            }
            ClassifyDestination::New {
                section_slug: slug,
                title: title.trim().to_string(),
            }
        } else {
            let Some(id) = form.collection_id.get_untracked() else {
                return;
            };
            ClassifyDestination::Existing(id)
        };

        cancel_pending.set(false);
        let season = parse_season(&form.season.get_untracked());
        let files = if sel.len() == 1 {
            vec![ClassifyInput {
                rel_path: sel[0].rel_path.clone(),
                title: form.title.get_untracked(),
            }]
        } else {
            sel.into_iter()
                .map(|e| ClassifyInput {
                    title: file_stem(&e.name),
                    rel_path: e.rel_path,
                })
                .collect()
        };
        job.dispatch(files, dest, season);
    });

    view! {
        {move || {
            if !open.get() || selection.get().is_empty() {
                return ().into_any();
            }
            let modal_title = if selection.get().len() == 1 {
                "Classify file"
            } else {
                "Classify files"
            };
            view! {
                <ModalShell
                    title=modal_title
                    subtitle=subtitle
                    busy=is_busy
                    on_close=try_close
                >
                    <Show when=move || show_setup.get()>
                        <div class="space-y-4">
                            <Show when=move || is_single.get()>
                                <TitleField value=form.title placeholder="Item title"/>
                            </Show>
                            <Show when=move || !is_single.get()>
                                <div class="text-xs text-gray-500 bg-white/5 \
                                                border border-white/10 rounded-lg px-3 py-2">
                                    "Each file keeps its own name as the item title."
                                </div>
                            </Show>
                            <SectionSelect
                                options={Signal::derive(move || sections.get().and_then(|r| r.ok()).unwrap_or_default())}
                                value=form.section_slug
                            />
                            <Show when=move || !form.section_slug.get().is_empty()>
                                <CollectionPicker
                                    section=chosen_section
                                    collection_id=form.collection_id
                                    creating_new=form.creating_new
                                    new_title=form.new_title
                                />
                            </Show>
                            <Show when=move || chosen_is_series.get()>
                                <SeasonField value=form.season/>
                            </Show>
                            <FormActions
                                on_cancel=try_close
                                on_confirm=confirm
                                confirm_label=confirm_label
                                disabled=Signal::derive(move || !can_move.get())
                            />
                        </div>
                    </Show>
                    <Show when=move || show_running.get()>
                        <BatchProgress
                            tracker=tracker
                            on_cancel=on_cancel
                            cancel_pending=cancel_pending.into()
                        />
                    </Show>
                    <Show when=move || show_final.get()>
                        <PhaseFinal
                            phase=phase
                            tracker=tracker
                            success_heading=success_heading
                            success_detail=success_detail
                            link=link
                            on_close=try_close
                        />
                    </Show>
                </ModalShell>
            }.into_any()
        }}
    }
}

#[component]
fn CollectionPicker(
    section: Signal<Option<SectionWithCollections>>,
    collection_id: RwSignal<Option<Uuid>>,
    creating_new: RwSignal<bool>,
    new_title: RwSignal<String>,
) -> impl IntoView {
    let create_error: Signal<Option<AppError>> = Signal::derive(|| None);

    view! {
        <div>
            <div class="flex items-center justify-between mb-1">
                <label class="text-xs text-gray-500 uppercase tracking-wider">"Collection"</label>
                <Show when=move || !creating_new.get()>
                    <button type="button"
                        on:click=move |_| creating_new.set(true)
                        class="text-xs px-2 py-0.5 rounded-md text-cyan-300 hover:text-white \
                                   bg-cyan-500/10 hover:bg-cyan-500/25 border border-cyan-400/30 \
                                   transition whitespace-nowrap">
                        "+ New collection"
                    </button>
                </Show>
            </div>

            <Show when=move || !creating_new.get()>
                {move || match section.get() {
                    None => ().into_any(),
                    Some(s) if s.collections.is_empty() => view! {
                        <div class="text-sm text-gray-500 bg-white/5 border border-white/10 \
                                        rounded-lg px-3 py-2">
                            "No collections yet — click “+ New collection” above."
                        </div>
                    }.into_any(),
                    Some(s) => view! {
                        <select
                            prop:value=move || collection_id.get().map(|u| u.to_string()).unwrap_or_default()
                            on:change=move |e| collection_id.set(Uuid::parse_str(&event_target_value(&e)).ok())
                            class="w-full bg-white/10 text-white rounded-lg px-3 py-1.5 text-sm \
                                       focus:outline-none focus:ring-1 focus:ring-cyan-400">
                            <option value="">"Choose a collection…"</option>
                            {s.collections.into_iter().map(|c| view! {
                                <option value=c.id.to_string()>{c.title.clone()}</option>
                            }).collect_view()}
                        </select>
                    }.into_any(),
                }}
            </Show>

            <Show when=move || creating_new.get()>
                <div class="space-y-2">
                    <div class="flex items-stretch gap-2">
                        <input type="text"
                            prop:value=move || new_title.get()
                            on:input=move |e| new_title.set(event_target_value(&e))
                            autofocus=true
                            placeholder="New collection title"
                            class="flex-1 bg-white/10 text-white rounded-lg px-3 py-1.5 text-sm \
                                       focus:outline-none focus:ring-1 focus:ring-cyan-400"/>
                        <button type="button"
                            on:click=move |_| creating_new.set(false)
                            class="shrink-0 px-3 rounded-lg bg-white/5 hover:bg-white/15 \
                                       text-gray-300 text-sm transition">
                            "Cancel"
                        </button>
                    </div>
                    <ErrorView error=create_error/>
                </div>
            </Show>
        </div>
    }
}
