//! The [`Playlist`] component and its supporting types.

use leptos::prelude::*;

use crate::{ContextBundle, MediaItem, Uuid};

mod row;

use row::PlaylistItem;

// ─── Edit mode ────────────────────────────────────────────────────────────
// Local to the playlist: when true, per-row rename / delete / move
// affordances are shown. The host passes its own signal through
// `Playlist`'s `edit_mode` prop; it is provided to the subtree here.

#[derive(Clone, Copy)]
struct EditMode(Signal<bool>);

pub(crate) fn use_edit_mode() -> Signal<bool> {
    expect_context::<EditMode>().0
}

// ─── Playlist config ──────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub(crate) struct PlaylistConfig {
    pub on_rename: Option<Callback<(Uuid, String)>>,
    pub on_delete: Option<Callback<Uuid>>,
    pub on_move: Option<Callback<(Uuid, bool)>>, // (id, is_up)
    pub show_download: bool,
}

impl ContextBundle for PlaylistConfig {}

// ─── Public component ─────────────────────────────────────────────────────

/// A scrollable sidebar listing every item in the playlist.
///
/// Shares `items` and `current_idx` with the [`MediaPlayer`](crate::MediaPlayer)
/// it is paired with. Renders an `<aside>` sized for a right-hand column;
/// the caller decides whether to show it (e.g. hide when `items.len() <= 1`).
#[component]
pub fn Playlist(
    items: Signal<Vec<MediaItem>>,
    current_idx: RwSignal<usize>,
    #[prop(into)] edit_mode: Signal<bool>,
    #[prop(default = None)] title: Option<String>,
    #[prop(default = None)] on_rename: Option<Callback<(Uuid, String)>>,
    #[prop(default = None)] on_delete: Option<Callback<Uuid>>,
    #[prop(default = None)] on_move: Option<Callback<(Uuid, bool)>>,
    #[prop(default = true)] show_download: bool,
) -> impl IntoView {
    provide_context(EditMode(edit_mode));
    PlaylistConfig::provide(PlaylistConfig {
        on_rename,
        on_delete,
        on_move,
        show_download,
    });

    let title = title.unwrap_or_else(|| "Playlist".to_string());

    view! {
        <aside class="w-full shrink-0 box-border font-sans text-white \
                      lg:w-72 xl:w-80 2xl:w-96">
            <div class="bg-white/5 backdrop-blur-sm \
                        rounded-2xl border border-white/10 \
                        overflow-hidden flex flex-col text-white \
                        max-h-[50vh] sm:max-h-[60vh] lg:max-h-[75vh]">
                <div class="px-4 py-3 border-b border-white/10 \
                            flex items-center justify-between shrink-0">
                    <h3 class="text-sm font-bold whitespace-nowrap overflow-hidden \
                               text-ellipsis m-0">
                        {title}
                    </h3>
                    <span class="text-xs text-gray-400 font-mono \
                                 bg-white/10 px-2 rounded-full leading-5">
                        {move || items.get().len()}
                    </span>
                </div>
                <div class="overflow-y-auto p-2 flex-1 min-h-0">
                    <For
                        each=move || {
                            let v = items.get();
                            let n = v.len();
                            v.into_iter().enumerate().map(move |(i, item)| (i, n, item))
                        }
                        key=|(_, _, item)| item.id
                        let:((index, total, item))
                    >
                        <PlaylistItem total item index current_idx/>
                    </For>
                </div>
            </div>
        </aside>
    }
}
