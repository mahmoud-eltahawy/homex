use leptos::prelude::*;

use crate::{MediaItem, MediaPlayer, Playlist, Uuid, style::stylesheet};

/// A ready-made watch page: [`MediaPlayer`] and [`Playlist`] composed in a
/// YouTube-style layout — player on the left, playlist column on the right,
/// stacking vertically on narrow screens.
///
/// Unlike [`MediaPlayer`] and [`Playlist`] — which take the playlist cursor
/// as a shared `RwSignal<usize>` so they can be wired together however you
/// like — `WatchPage` **owns** its cursor. Pass the playlist and an optional
/// starting index; the page creates the signal internally and hands it to
/// both children.
///
/// `initial_idx` is just that: initial. Changing it after mount has no
/// effect, and the cursor is not exposed upward. If you need to observe or
/// drive the cursor from outside (e.g. persist it, sync it to the URL, or
/// share it with a third component), compose [`MediaPlayer`] and
/// [`Playlist`] yourself with your own signal — see the crate-level docs
/// for an example.
///
/// The playlist is hidden automatically when there is only one item, since
/// an "up next" column with a single entry is noise.
#[component]
pub fn WatchPage(
    items: Signal<Vec<MediaItem>>,
    /// Index to select on mount. Clamped to `items.len() - 1`; ignored (and
    /// treated as `0`) when `items` is empty.
    #[prop(default = 0)]
    initial_idx: usize,
    #[prop(into)] edit_mode: Signal<bool>,
    #[prop(default = false)] audio: bool,
    #[prop(default = None)] artwork: Option<String>,
    #[prop(optional)] playlist_title: Option<String>,
    #[prop(optional)] on_rename: Option<Callback<(Uuid, String)>>,
    #[prop(optional)] on_delete: Option<Callback<Uuid>>,
    #[prop(optional)] on_move: Option<Callback<(Uuid, bool)>>,
    #[prop(default = true)] show_download: bool,
) -> impl IntoView {
    // Clamp the starting index up front so a too-large `initial_idx` doesn't
    // flash an empty selection for a frame before `MediaPlayer`'s clamp
    // effect kicks in.
    let len = items.get_untracked().len();
    let start = if len == 0 {
        0
    } else {
        initial_idx.min(len - 1)
    };
    let current_idx = RwSignal::new(start);

    let show_playlist = Signal::derive(move || items.get().len() > 1);
    let title_for_playlist = playlist_title.clone();

    view! {
        {stylesheet()}
        <div class="lmp-watch">
            <div class="lmp-watch-main">
                <MediaPlayer
                    items=items
                    current_idx=current_idx
                    audio=audio
                    artwork=artwork
                />
            </div>
            <Show when=move || show_playlist.get()>
                <Playlist
                    items=items
                    current_idx=current_idx
                    edit_mode=edit_mode
                    title=title_for_playlist.clone()
                    on_rename=on_rename
                    on_delete=on_delete
                    on_move=on_move
                    show_download=show_download
                />
            </Show>
        </div>
    }
}
