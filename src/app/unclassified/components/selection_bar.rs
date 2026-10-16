use leptos::prelude::*;

use crate::app::unclassified::helpers::{kind_for_ext, plural, select_all};
use crate::app::unclassified::types::UnclassifiedEntry;

#[component]
pub fn SelectionBar(
    entries: Vec<UnclassifiedEntry>,
    file_selection: RwSignal<Vec<UnclassifiedEntry>>,
    folder_selection: RwSignal<Vec<UnclassifiedEntry>>,
    modal_open: RwSignal<bool>,
    folder_modal_open: RwSignal<bool>,
) -> impl IntoView {
    let total_supported = entries
        .iter()
        .filter(|e| !e.is_dir && kind_for_ext(e.ext.as_ref()).is_some())
        .count();

    let all_files_selected = move || {
        let n = file_selection.get().len();
        total_supported > 0 && n == total_supported
    };

    let on_select_all = {
        let entries = entries.clone();
        Callback::new(move |ev: web_sys::Event| {
            if event_target_checked(&ev) {
                select_all(file_selection, entries.clone());
            } else {
                file_selection.set(Vec::new());
            }
        })
    };

    let file_count = move || file_selection.get().len();
    let folder_count = move || folder_selection.get().len();
    let total_count = move || file_count() + folder_count();

    let on_clear_all = Callback::new(move |_: ()| {
        file_selection.set(Vec::new());
        folder_selection.set(Vec::new());
    });

    let on_classify = Callback::new(move |_: ()| {
        let files = file_count();
        let folders = folder_count();
        if folders > 0 {
            folder_modal_open.set(true);
        } else if files > 0 {
            modal_open.set(true);
        }
    });

    view! {
        <div class="flex items-center justify-between gap-3 mb-3 px-1 min-h-9">
            <div class="flex items-center gap-3 min-w-0">
                <Show when=move || { total_supported > 0 }>
                    <label class="flex items-center gap-2 cursor-pointer select-none">
                        <input type="checkbox"
                            prop:checked=all_files_selected
                            on:change=move |ev| on_select_all.run(ev)
                            class="accent-cyan-400 w-4 h-4"/>
                        <span class="text-xs text-gray-400">"Select all files"</span>
                    </label>
                </Show>
                {move || (total_count() > 0).then(|| view! {
                    <span class="text-xs text-cyan-300 font-medium">
                        {move || {
                            let f = file_count();
                            let d = folder_count();
                            match (f, d) {
                                (0, d) => format!("{d} folder{}", plural(d)),
                                (f, 0) => format!("{f} file{}", plural(f)),
                                (f, d) => format!("{f} file{}, {d} folder{}", plural(f), plural(d)),
                            }
                        }}
                    </span>
                })}
            </div>
            <div class="flex items-center gap-2 shrink-0">
                {move || (total_count() > 0).then(|| view! {
                    <>
                        <button type="button"
                            on:click=move |_| on_clear_all.run(())
                            class="text-xs px-2 py-1 rounded-lg text-gray-300 hover:bg-white/10 transition">
                            "Clear"
                        </button>
                        <button type="button"
                            on:click=move |_| on_classify.run(())
                            class="text-xs px-3 py-1.5 rounded-lg \
                                       bg-gradient-to-r from-cyan-500 to-blue-500 \
                                       hover:from-cyan-400 hover:to-blue-400 \
                                       text-white font-bold transition">
                            {move || {
                                let f = file_count();
                                let d = folder_count();
                                if d > 0 && f == 0 {
                                    format!("Classify {d} folder{}", plural(d))
                                } else if f > 0 && d == 0 {
                                    format!("Classify {f} file{}", plural(f))
                                } else {
                                    format!("Classify {} item{}", f + d, plural(f + d))
                                }
                            }}
                        </button>
                    </>
                })}
            </div>
        </div>
    }
}
