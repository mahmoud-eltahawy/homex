use leptos::either::Either;
use leptos::prelude::*;

use crate::app::unclassified::helpers::{
    format_size, kind_for_ext, toggle_folder_selection, toggle_selection,
};
use crate::app::unclassified::types::UnclassifiedEntry;

use super::icons::{FileIcon, FolderIcon};

#[component]
pub fn DirRow(
    entry: UnclassifiedEntry,
    on_navigate: Callback<String>,
    folder_selection: RwSignal<Vec<UnclassifiedEntry>>,
) -> impl IntoView {
    let target_open = entry.rel_path.clone();
    let target_arrow = entry.rel_path.clone();
    let name = entry.name.clone();
    let key = entry.rel_path.clone();
    let entry_for_toggle = entry.clone();
    let entry_for_single = entry;

    let is_selected =
        Signal::derive(move || folder_selection.get().iter().any(|e| e.rel_path == key));

    let on_toggle = Callback::new(move |ev: web_sys::Event| {
        if event_target_checked(&ev) {
            toggle_folder_selection(folder_selection, entry_for_toggle.clone());
        } else {
            let path = entry_for_toggle.rel_path.clone();
            folder_selection.update(|v| v.retain(|e| e.rel_path != path));
        }
    });

    let on_single_classify = Callback::new(move |_: ()| {
        folder_selection.set(vec![entry_for_single.clone()]);
    });

    view! {
        <div class="flex items-center gap-2 pr-3 hover:bg-white/5 transition">
            <input type="checkbox"
                prop:checked=move || is_selected.get()
                on:change=move |ev| on_toggle.run(ev)
                class="accent-cyan-400 w-4 h-4 shrink-0 cursor-pointer ml-4"/>
            <button type="button"
                on:click=move |_| on_navigate.run(target_open.clone())
                class="flex-1 text-left flex items-center gap-3 px-2 py-2.5 min-w-0">
                <span class="text-cyan-400 shrink-0"><FolderIcon/></span>
                <span class="text-white text-sm truncate flex-1">{name}</span>
            </button>
            <button type="button"
                on:click=move |_| on_single_classify.run(())
                class="text-xs shrink-0 px-2 py-1 rounded-md text-cyan-300 hover:text-white \
                           bg-cyan-500/10 hover:bg-cyan-500/25 border border-cyan-400/30 \
                           transition whitespace-nowrap">
                "Classify"
            </button>
            <button type="button"
                on:click=move |_| on_navigate.run(target_arrow.clone())
                class="text-gray-500 text-xs shrink-0 hover:text-white transition">
                "Open →"
            </button>
        </div>
    }
}

#[component]
pub fn FileRow(
    entry: UnclassifiedEntry,
    file_selection: RwSignal<Vec<UnclassifiedEntry>>,
    modal_open: RwSignal<bool>,
) -> impl IntoView {
    let is_supported = kind_for_ext(entry.ext.as_deref()).is_some();
    let name = entry.name.clone();
    let size = entry.size;
    let key = entry.rel_path.clone();

    let is_selected =
        Signal::derive(move || file_selection.get().iter().any(|e| e.rel_path == key));

    let on_change = {
        let entry = entry.clone();
        Callback::new(move |ev: web_sys::Event| {
            if event_target_checked(&ev) {
                toggle_selection(file_selection, entry.clone());
            } else {
                file_selection.update(|v| v.retain(|e| e.rel_path != entry.rel_path));
            }
        })
    };

    let on_single_classify = Callback::new(move |_: ()| {
        file_selection.set(vec![entry.clone()]);
        modal_open.set(true);
    });

    let row_class = if is_supported {
        "flex items-center gap-3 px-4 py-2.5 transition hover:bg-white/5"
    } else {
        "flex items-center gap-3 px-4 py-2.5 opacity-50"
    };

    view! {
        <div class=row_class>
            <input type="checkbox"
                disabled=!is_supported
                prop:checked=move || is_selected.get()
                on:change=move |ev| on_change.run(ev)
                class="accent-cyan-400 w-4 h-4 shrink-0 cursor-pointer disabled:cursor-not-allowed"/>
            <span class="text-gray-400 shrink-0"><FileIcon/></span>
            <span class="text-white text-sm truncate flex-1">{name}</span>
            <span class="text-gray-500 text-xs font-mono shrink-0">{format_size(size)}</span>
            {if is_supported {
                Either::Left(view! {
                    <button type="button"
                        on:click=move |_| on_single_classify.run(())
                        class="text-xs shrink-0 text-cyan-400 hover:text-cyan-300 transition">
                        "Classify →"
                    </button>
                })
            } else {
                Either::Right(view! {
                    <span class="text-xs shrink-0 text-gray-600">"Unsupported"</span>
                })
            }}
        </div>
    }
}
