//! A single playlist row and its state indicator.
//!
//! Owns per-row interaction (select, rename, delete, reorder) but not the
//! surrounding panel — that's [`super::Playlist`].

use leptos::either::Either;
use leptos::html;
use leptos::prelude::*;

use crate::ContextBundle;
use crate::MediaItem;
use icons::{DeleteIcon, DownloadIcon, EditIcon, player::PlayIcon};

use super::{PlaylistConfig, use_edit_mode};

// ─── Shared class fragments ───────────────────────────────────────────────

/// The row surface, minus the current/non-current variants which are
/// applied by a class closure.
const ROW_BASE: &str = "group w-full flex items-center gap-2 p-2 rounded-lg border cursor-pointer \
     text-start transition-colors duration-150 box-border \
     motion-reduce:transition-none";

const ROW_CURRENT: &str = "bg-cyan-400/15 border-cyan-400/30 hover:bg-cyan-400/20";
const ROW_IDLE: &str = "border-transparent hover:bg-white/5";

/// A row action (download, rename, delete). Hidden until the row is hovered,
/// always visible on coarse pointers.
const ROW_ACTION: &str = "inline-flex items-center justify-center text-gray-400 p-1 shrink-0 \
     opacity-0 group-hover:opacity-100 \
     transition-[opacity,color] duration-150 \
     no-underline bg-transparent border-none cursor-pointer \
     hover:text-white motion-reduce:transition-none \
     [@media(pointer:coarse)]:p-2 \
     [@media(pointer:coarse)]:opacity-100";

/// The reorder arrows in edit mode.
const ROW_ARROW: &str = "text-gray-500 bg-transparent border-none cursor-pointer px-1 leading-none text-xs \
     transition-colors hover:text-white disabled:opacity-20 disabled:cursor-not-allowed";

// ─── Row ──────────────────────────────────────────────────────────────────

#[component]
pub(crate) fn PlaylistItem(
    item: MediaItem,
    index: usize,
    total: usize,
    current_idx: RwSignal<usize>,
) -> impl IntoView {
    let config = PlaylistConfig::expect();

    let id = item.id;
    let download_src = item.src.clone();
    let download_name = item.title.clone();

    let title = StoredValue::new(item.title.clone());
    let subtitle = StoredValue::new(item.subtitle.clone());

    let is_current = move || current_idx.get() == index;
    let is_first = index == 0;
    let is_last = index + 1 == total;

    let editing = RwSignal::new(false);
    let draft = RwSignal::new(item.title.clone());
    let input_ref = NodeRef::<html::Input>::new();

    let on_select = move |_| {
        if !editing.get_untracked() {
            current_idx.set(index);
        }
    };

    let begin_edit = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        draft.set(title.get_value());
        editing.set(true);
    };

    let do_commit: Callback<()> = Callback::new(move |_| {
        editing.set(false);
        let new_title = draft.get_untracked();
        if new_title != title.get_value()
            && let Some(cb) = config.on_rename
        {
            cb.run((id, new_title));
        }
    });

    let on_delete_click = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        if let Some(cb) = config.on_delete {
            cb.run(id);
        }
    };

    let on_move_click = move |ev: web_sys::MouseEvent, up: bool| {
        ev.stop_propagation();
        if let Some(cb) = config.on_move {
            cb.run((id, up));
        }
    };

    Effect::new(move |_| {
        if editing.get()
            && let Some(input) = input_ref.get()
        {
            let _ = input.focus();
            input.select();
        }
    });

    let row_class = move || {
        if is_current() {
            format!("{ROW_BASE} {ROW_CURRENT}")
        } else {
            format!("{ROW_BASE} {ROW_IDLE}")
        }
    };

    let edit_on = use_edit_mode();
    let can_rename = config.on_rename.is_some();
    let can_delete = config.on_delete.is_some();
    let can_move = config.on_move.is_some();
    let show_download = config.show_download;

    view! {
        <div class=row_class on:click=on_select>
            <Show when=move || !(edit_on.get() && can_move)>
                <PlaylistIndicator index=index is_current=Signal::derive(is_current)/>
            </Show>

            <Show when=move || edit_on.get() && can_move>
                <div class="flex flex-col shrink-0">
                    <button
                        type="button"
                        class=ROW_ARROW
                        disabled=is_first
                        on:click=move |ev| on_move_click(ev, true)
                        aria-label="Move up"
                    >
                        "▲"
                    </button>
                    <button
                        type="button"
                        class=ROW_ARROW
                        disabled=is_last
                        on:click=move |ev| on_move_click(ev, false)
                        aria-label="Move down"
                    >
                        "▼"
                    </button>
                </div>
            </Show>

            <div class="flex-1 min-w-0">
                <Show
                    when=move || editing.get()
                    fallback=move || view! {
                        <>
                            <div class="text-sm text-white whitespace-nowrap \
                                        overflow-hidden text-ellipsis">
                                {title.get_value()}
                            </div>
                            {subtitle.get_value().map(|s| view! {
                                <div class="text-xs text-gray-400 whitespace-nowrap \
                                            overflow-hidden text-ellipsis">
                                    {s}
                                </div>
                            })}
                        </>
                    }
                >
                    <input
                        node_ref=input_ref
                        type="text"
                        class="w-full box-border bg-white/10 text-white text-sm \
                               rounded border-none outline-none px-2 py-1 font-inherit \
                               focus:ring-1 focus:ring-cyan-400"
                        prop:value=move || draft.get()
                        on:input=move |ev| draft.set(event_target_value(&ev))
                        on:keydown=move |ev: web_sys::KeyboardEvent| match ev.key().as_str() {
                            "Enter" => { ev.prevent_default(); do_commit.run(()); }
                            "Escape" => { ev.prevent_default(); editing.set(false); }
                            _ => {}
                        }
                        on:blur=move |_| do_commit.run(())
                        on:click=move |ev| ev.stop_propagation()
                    />
                </Show>
            </div>

            <Show when=move || show_download>
                <a
                    href=download_src.clone()
                    download=download_name.clone()
                    class=ROW_ACTION
                    aria-label="Download"
                    on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                >
                    <DownloadIcon/>
                </a>
            </Show>

            <Show when=move || edit_on.get() && can_rename>
                <button
                    type="button"
                    class=ROW_ACTION
                    on:click=begin_edit
                    aria-label="Rename"
                >
                    <EditIcon/>
                </button>
            </Show>

            <Show when=move || edit_on.get() && can_delete>
                <button
                    type="button"
                    class=format!("{ROW_ACTION} hover:text-red-300")
                    on:click=on_delete_click
                    aria-label="Delete"
                >
                    <DeleteIcon/>
                </button>
            </Show>
        </div>
    }
}

// ─── Row parts ────────────────────────────────────────────────────────────

#[component]
fn PlaylistIndicator(index: usize, #[prop(into)] is_current: Signal<bool>) -> impl IntoView {
    view! {
        {move || if is_current.get() {
            Either::Left(view! {
                <span class="flex items-center justify-center w-8 h-8 rounded-full \
                             shrink-0 bg-cyan-400/15 text-cyan-400">
                    <PlayIcon/>
                </span>
            })
        } else {
            Either::Right(view! {
                <span class="flex items-center justify-center w-8 h-8 rounded-full \
                             shrink-0 bg-white/5 text-gray-300 text-xs font-bold">
                    {index + 1}
                </span>
            })
        }}
    }
}
