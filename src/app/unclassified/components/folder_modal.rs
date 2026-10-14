use crate::app::error::AppError;
use crate::app::unclassified::helpers::{format_size, plural};
use crate::app::unclassified::jobs::FolderClassifyJob;
use crate::app::unclassified::tracker::BatchPhase;
use crate::app::unclassified::types::{
    ClassifyDestination, DestinationMode, FoldersInspection, SectionWithCollections,
    UnclassifiedEntry,
};
use crate::app::unclassified::{fetch_sections_with_collections, inspect_unclassified_folders};
use leptos::prelude::*;
use leptos_outcome::ErrorView;
use uuid::Uuid;

use super::primitives::{FormActions, ModalShell, SectionSelect, TitleField};
use super::progress::{BatchProgress, PhaseFinal};

#[component]
pub fn FolderClassifyModal(
    selection: RwSignal<Vec<UnclassifiedEntry>>,
    open: RwSignal<bool>,
    job: FolderClassifyJob,
    on_close: Callback<()>,
) -> impl IntoView {
    let inspection: Resource<Result<FoldersInspection, AppError>> = Resource::new(
        move || {
            if !open.get() {
                return None;
            }
            Some(
                selection
                    .get()
                    .iter()
                    .map(|e| e.rel_path.clone())
                    .collect::<Vec<_>>(),
            )
        },
        |paths| async move {
            match paths {
                Some(p) if !p.is_empty() => inspect_unclassified_folders(p).await,
                _ => Err(AppError::public("No folder selected")),
            }
        },
    );

    let common_kind = Memo::new(move |_| {
        inspection
            .get()
            .and_then(|r| r.ok())
            .and_then(|i| i.common_kind)
    });

    let sections: Resource<Result<Vec<SectionWithCollections>, AppError>> = Resource::new(
        move || {
            if !open.get() {
                return None;
            }
            let kind = common_kind.get().map(|k| k.as_str().to_string())?;
            Some((selection.get().first()?.rel_path.clone(), kind))
        },
        |input| async move {
            match input {
                Some((_, kind)) => fetch_sections_with_collections(kind).await,
                None => Ok(Vec::new()),
            }
        },
    );

    let mode = RwSignal::new(DestinationMode::New);
    let section_slug = RwSignal::new(String::new());
    let section_title = RwSignal::new(String::new());

    Effect::new(move |_| {
        let _ = open.get();
        mode.set(DestinationMode::New);
        section_slug.set(String::new());
        section_title.set(String::new());
    });

    Effect::new(move |_| {
        if let Some(Ok(insp)) = inspection.get()
            && section_title.get_untracked().trim().is_empty()
            && let Some(first) = insp.folders.first()
        {
            section_title.set(first.name.clone());
        }
    });

    Effect::new(move |_| {
        if !open.get() {
            job.reset();
        }
    });

    let existing_options: Signal<Vec<SectionWithCollections>> = Signal::derive(move || {
        let nested_only = inspection
            .get()
            .and_then(|r| r.ok())
            .map(|i| i.any_has_subdirs)
            .unwrap_or(false);
        sections
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
            .into_iter()
            .filter(|s| !nested_only || s.nested)
            .collect()
    });

    let has_existing = Signal::derive(move || !existing_options.get().is_empty());

    Effect::new(move |_| {
        if inspection.get().is_some() && !has_existing.get() {
            mode.set(DestinationMode::New);
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

    let subtitle = Signal::derive(move || {
        let sel = selection.get();
        match sel.len() {
            0 => String::new(),
            1 => sel[0].name.clone(),
            n => format!("{n} folders"),
        }
    });

    let can_confirm = Signal::derive(move || {
        if !matches!(inspection.get(), Some(Ok(ref i)) if i.common_kind.is_some()) {
            return false;
        }
        match mode.get() {
            DestinationMode::Existing => !section_slug.get().is_empty(),
            DestinationMode::New => !section_title.get().trim().is_empty(),
        }
    });

    let confirm_label = Signal::derive(move || {
        let n = selection.get().len();
        if n <= 1 {
            "Classify folder".to_string()
        } else {
            format!("Classify {n} folders")
        }
    });

    let success_heading = Signal::derive(|| "Folders classified".to_string());
    let success_detail = Signal::derive(move || {
        let files = tracker.slots().get().len();
        format!("{files} file{} moved.", plural(files))
    });

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
        let paths: Vec<String> = sel.iter().map(|e| e.rel_path.clone()).collect();
        let dest = match mode.get_untracked() {
            DestinationMode::Existing => {
                let slug = section_slug.get_untracked();
                if slug.is_empty() {
                    return;
                }
                // Existing sections are handled by the "folder → existing
                // section" branch on the server, which needs a section slug.
                // We wrap it as a New with the existing section_slug... no —
                // the classify server fn accepts `ClassifyDestination::New`
                // with the section's title, but here we only have the slug.
                // Use the section picker's slug and pass it as New's
                // section_slug + the section's title.
                let title = existing_options
                    .get_untracked()
                    .into_iter()
                    .find(|s| s.slug == slug)
                    .map(|s| s.title)
                    .unwrap_or_default();
                ClassifyDestination::New {
                    section_slug: slug,
                    title,
                }
            }
            DestinationMode::New => {
                let t = section_title.get_untracked();
                let t = t.trim();
                if t.is_empty() {
                    return;
                }
                let slug = section_slug.get_untracked();
                if slug.is_empty() {
                    return;
                }
                ClassifyDestination::New {
                    section_slug: slug,
                    title: t.to_string(),
                }
            }
        };
        cancel_pending.set(false);
        job.dispatch(paths, dest);
    });

    view! {
        {move || {
            if !open.get() || selection.get().is_empty() {
                return ().into_any();
            }
            let modal_title = if selection.get().len() == 1 {
                "Classify folder"
            } else {
                "Classify folders"
            };
            view! {
                <ModalShell
                    title=modal_title
                    subtitle=subtitle
                    busy=is_busy
                    on_close=try_close
                >
                    <Show when=move || show_setup.get()>
                        <FolderSetup
                            inspection=inspection
                            existing_options=existing_options
                            has_existing=has_existing
                            mode
                            section_slug
                            section_title
                            can_confirm=can_confirm
                            confirm_label=confirm_label
                            on_cancel=try_close
                            on_confirm=confirm
                        />
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
fn FolderSetup(
    inspection: Resource<Result<FoldersInspection, AppError>>,
    existing_options: Signal<Vec<SectionWithCollections>>,
    has_existing: Signal<bool>,
    mode: RwSignal<DestinationMode>,
    section_slug: RwSignal<String>,
    section_title: RwSignal<String>,
    can_confirm: Signal<bool>,
    confirm_label: Signal<String>,
    on_cancel: Callback<()>,
    on_confirm: Callback<()>,
) -> impl IntoView {
    view! {
        {move || match inspection.get() {
            None => view! {
                <div class="py-8 text-center text-gray-500 text-sm">"Inspecting folders…"</div>
            }.into_any(),
            Some(Err(e)) => view! {
                <div class="py-4">
                    <ErrorView
                        error=Signal::derive(move || Some(e.clone()))
                        banner=true
                        heading="Cannot classify"/>
                </div>
            }.into_any(),
            Some(Ok(insp)) if insp.common_kind.is_none() => view! {
                <div class="py-4 space-y-3">
                    <ErrorView
                        error=Signal::derive(|| Some(AppError::public(
                            "The selected folders are empty or mix video and audio. \
                                 Select folders of the same kind and try again."
                        )))
                        banner=true
                        heading="Cannot classify"/>
                    <div class="flex justify-end">
                        <button type="button"
                            on:click=move |_| on_cancel.run(())
                            class="px-4 py-2 rounded-lg bg-white/10 hover:bg-white/20 \
                                       text-white text-sm transition">
                            "Close"
                        </button>
                    </div>
                </div>
            }.into_any(),
            Some(Ok(insp)) => {
                let kind = insp.common_kind.unwrap();
                let folder_count = insp.folders.len();
                let total_files = insp.total_files;
                let any_has_subdirs = insp.any_has_subdirs;
                let total_size = insp.total_size;
                let kind_label = kind.label().to_string();

                let mut collections: Vec<String> = Vec::new();
                for f in &insp.folders {
                    if f.has_subdirs {
                        collections.extend(f.subdir_names.iter().cloned());
                    } else {
                        collections.push(f.name.clone());
                    }
                }
                let collection_count = collections.len();
                let collections_preview = StoredValue::new(collections);

                let summary = format!(
                    "{} folder{} · {} file{} · {} · {} · {}",
                    folder_count,
                    plural(folder_count),
                    total_files,
                    plural(total_files),
                    format_size(total_size),
                    kind_label,
                    if any_has_subdirs { "nested" } else { "non-nested" },
                );

                view! {
                    <div class="space-y-4">
                        <div class="text-xs text-gray-400 bg-white/5 border border-white/10 \
                                        rounded-lg px-3 py-2">
                            <div class="text-white text-sm font-medium mb-0.5">{summary}</div>
                            <div>{if any_has_subdirs {
                                "Each subfolder becomes its own collection. Only series / group sections are valid destinations."
                            } else {
                                "Each folder becomes its own collection."
                            }}</div>
                        </div>

                        <DestinationToggle mode has_existing/>

                        <Show when=move || mode.get() == DestinationMode::New>
                            <div class="space-y-3">
                                <TitleField label="Section title" value=section_title placeholder="Section title"/>
                                <Show when=move || { collection_count > 1 || any_has_subdirs }>
                                    <div class="text-xs text-gray-400 bg-white/5 \
                                                    border border-white/10 rounded-lg px-3 py-2 \
                                                    max-h-40 overflow-auto">
                                        <div class="text-gray-500 uppercase tracking-wider mb-1">
                                            {move || format!(
                                                "{collection_count} new collection{}",
                                                plural(collection_count),
                                            )}
                                        </div>
                                        <ul class="space-y-0.5">
                                            {move || collections_preview.with_value(|v| {
                                                v.iter().map(|name| view! {
                                                    <li class="truncate text-white">"• " {name.clone()}</li>
                                                }).collect_view()
                                            })}
                                        </ul>
                                    </div>
                                </Show>
                            </div>
                        </Show>

                        <Show when=move || mode.get() == DestinationMode::Existing>
                            <SectionSelect options=existing_options value=section_slug/>
                        </Show>

                        <FormActions
                            on_cancel=on_cancel
                            on_confirm=on_confirm
                            confirm_label=confirm_label
                            disabled=Signal::derive(move || !can_confirm.get())
                        />
                    </div>
                }.into_any()
            }
        }}
    }
}

#[component]
fn DestinationToggle(mode: RwSignal<DestinationMode>, has_existing: Signal<bool>) -> impl IntoView {
    let btn = move |this: DestinationMode, label: &'static str| {
        let class = move || {
            let active = mode.get() == this;
            if active {
                "text-xs px-3 py-1.5 rounded-md transition bg-cyan-500/25 text-white"
            } else {
                "text-xs px-3 py-1.5 rounded-md transition text-gray-400 hover:text-white"
            }
        };
        view! {
            <button type="button" on:click=move |_| mode.set(this) class=class>{label}</button>
        }
    };

    view! {
        <div>
            <div class="text-xs text-gray-500 uppercase tracking-wider mb-1">"Destination"</div>
            <div class="inline-flex rounded-lg bg-white/5 border border-white/10 p-0.5">
                {btn(DestinationMode::New, "New section")}
                <Show when=move || has_existing.get()>
                    {btn(DestinationMode::Existing, "Existing section")}
                </Show>
            </div>
        </div>
    }
}
