//! A single playlist row and its state indicator.
//!
//! Owns per-row interaction (select, rename, delete, reorder) but not the
//! surrounding panel — that's [`super::Playlist`].

use leptos::either::Either;
use leptos::html;
use leptos::prelude::*;

use crate::ContextBundle;
use crate::MediaItem;
use crate::icons::{DeleteIcon, DownloadIcon, EditIcon, PlayIcon};

use super::{PlaylistConfig, use_edit_mode};

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
            "lmp-row lmp-row--current"
        } else {
            "lmp-row"
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
                <div class="lmp-row-arrows">
                    <button
                        type="button"
                        class="lmp-row-arrow"
                        disabled=is_first
                        on:click=move |ev| on_move_click(ev, true)
                        aria-label="Move up"
                    >
                        "▲"
                    </button>
                    <button
                        type="button"
                        class="lmp-row-arrow"
                        disabled=is_last
                        on:click=move |ev| on_move_click(ev, false)
                        aria-label="Move down"
                    >
                        "▼"
                    </button>
                </div>
            </Show>

            <div class="lmp-row-text">
                <Show
                    when=move || editing.get()
                    fallback=move || view! {
                        <>
                            <div class="lmp-row-title">{title.get_value()}</div>
                            {subtitle.get_value().map(|s| view! {
                                <div class="lmp-row-subtitle">{s}</div>
                            })}
                        </>
                    }
                >
                    <input
                        node_ref=input_ref
                        type="text"
                        class="lmp-row-input"
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
                    class="lmp-row-action"
                    aria-label="Download"
                    on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                >
                    <DownloadIcon/>
                </a>
            </Show>

            <Show when=move || edit_on.get() && can_rename>
                <button
                    type="button"
                    class="lmp-row-action"
                    on:click=begin_edit
                    aria-label="Rename"
                >
                    <EditIcon/>
                </button>
            </Show>

            <Show when=move || edit_on.get() && can_delete>
                <button
                    type="button"
                    class="lmp-row-action lmp-row-action--delete"
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
                <span class="lmp-row-indicator lmp-row-indicator--current">
                    <PlayIcon/>
                </span>
            })
        } else {
            Either::Right(view! {
                <span class="lmp-row-indicator lmp-row-indicator--number">
                    {index + 1}
                </span>
            })
        }}
    }
}
