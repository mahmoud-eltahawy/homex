//! # leptos_media_player
//!
//! A small media-player component for [Leptos](https://leptos.dev), packaged
//! with a YouTube-style watch page and a scrollable playlist column.
//!
//! ## What's in the box
//!
//! | Item            | What it does                                                                                                                  |
//! |-----------------|-------------------------------------------------------------------------------------------------------------------------------|
//! | [`MediaPlayer`] | A `<video>` (or audio) surface with play/pause, seek, volume, mute, fullscreen, and prev/next controls.                        |
//! | [`Playlist`]    | A scrollable list of [`MediaItem`]s with optional edit-mode (rename / delete / reorder) and download links.                    |
//! | [`WatchPage`]   | A convenience wrapper that lays the two out YouTube-style: player on the left, playlist on the right. Stacks vertically on phones. |
//! | [`MediaItem`]   | The playlist entry type: `id`, `title`, optional `subtitle`, `src`, optional `artwork`.                                        |
//!
//! ## Choosing between `WatchPage` and the primitives
//!
//! [`MediaPlayer`] and [`Playlist`] are **primitives**: they take the playlist
//! cursor as a shared `RwSignal<usize>` and let you compose them however you
//! like — side by side, one above the other, two players sharing one list,
//! inside a modal, wired to a URL, wired to `localStorage`, etc. They know
//! nothing about each other.
//!
//! [`WatchPage`] is a **convenience composition**. It owns the cursor (you
//! pass `initial_idx`, it creates the signal internally), lays the two
//! children out in the default YouTube-style arrangement, and hides the
//! playlist when there's only one item.
//!
//! Reach for [`WatchPage`] when you want the default experience. Drop to
//! [`MediaPlayer`] + [`Playlist`] when you need to own or observe the cursor,
//! or when the layout isn't the default.
//!
//! ## Identifiers
//!
//! [`MediaItem::id`] is a [`Uuid`]. The crate re-exports `Uuid` so callers
//! don't need to depend on the `uuid` crate directly. The same UUID is the
//! key used for `For` iteration and the identifier passed to `on_rename` /
//! `on_delete` / `on_move`.
//!
//! ## Keyboard shortcuts
//!
//! Click anywhere on the player (or Tab into it) to focus it, then:
//!
//! | Key            | Action                    |
//! |----------------|---------------------------|
//! | `Space`, `k`   | Toggle play / pause       |
//! | `m`            | Toggle mute               |
//! | `f`            | Toggle fullscreen         |
//! | `←` / `→`      | Seek back / forward 5 s   |
//! | `↑` / `↓`      | Volume up / down 5 %      |
//! | `0` – `9`      | Jump to 0 % – 90 %        |
//! | `Home` / `End` | Jump to start / end       |
//!
//! Shortcuts are scoped to the player — they never fire page-wide, and they
//! stay out of the way while you type in a form field or nudge a slider.
//!
//! ## Styling
//!
//! The components are styled entirely with Tailwind utility classes. There is
//! no embedded stylesheet, no custom-property theming layer, and no external
//! CSS file to import. Restyle anything by editing the classes in the
//! component sources, or by wrapping the components in your own container and
//! targeting them with descendant selectors.
//!
//! The consuming app's Tailwind build must scan this crate's source. In a
//! Tailwind 4 setup, add to your stylesheet:
//!
//! ```css
//! @import "tailwindcss";
//! @source "../../crates/leptos_media_player/src";
//! ```
//!
//! Adjust the path to match your workspace layout.
//!
//! ## Responsive behaviour
//!
//! [`WatchPage`] stacks the player above the playlist on phones and switches
//! to a side-by-side layout at `lg` (`≥ 1024 px`). No JavaScript is involved
//! — it's a single Tailwind breakpoint.
//!
//! ## Usage
//!
//! ### Video playlist
//!
//! The common case: hand `WatchPage` the items and an edit-mode signal, and
//! it does the rest. The cursor starts at index `0` by default.
//!
//! ```rust,no_run
//! use leptos::prelude::*;
//! use leptos_media_player::{MediaItem, Uuid, WatchPage};
//!
//! #[component]
//! pub fn App() -> impl IntoView {
//!     let items = RwSignal::new(vec![
//!         MediaItem::new(Uuid::new_v4(), "Big Buck Bunny", "/media/bbb.mp4")
//!             .with_subtitle("2008 · 9:56"),
//!         MediaItem::new(Uuid::new_v4(), "Sintel", "/media/sintel.mp4")
//!             .with_subtitle("2010 · 14:48"),
//!         MediaItem::new(Uuid::new_v4(), "Tears of Steel", "/media/tos.mp4")
//!             .with_subtitle("2012 · 12:14"),
//!     ]);
//!     let edit_mode = RwSignal::new(false);
//!     let items_sig = Signal::derive(move || items.get());
//!
//!     view! {
//!         <WatchPage
//!             items=items_sig
//!             edit_mode=edit_mode
//!         />
//!     }
//! }
//! ```
//!
//! ### Starting on a specific item
//!
//! Pass `initial_idx` to open on a track other than the first. It's clamped
//! to the last valid index, so out-of-range values are safe.
//!
//! ```rust,no_run
//! # use leptos::prelude::*;
//! # use leptos_media_player::{MediaItem, WatchPage};
//! # #[component]
//! # fn App() -> impl IntoView {
//! # let items = RwSignal::new(Vec::<MediaItem>::new());
//! # let edit_mode = RwSignal::new(false);
//! # let items_sig = Signal::derive(move || items.get());
//! view! {
//!     <WatchPage
//!         items=items_sig
//!         initial_idx=2
//!         edit_mode=edit_mode
//!     />
//! }
//! # }
//! ```
//!
//! ### Audio with a default artwork
//!
//! Set `audio = true` and pass a default `artwork` image URL. Each
//! [`MediaItem`] can also carry its own `artwork`, which wins over the
//! component-level fallback.
//!
//! ```rust,no_run
//! # use leptos::prelude::*;
//! # use leptos_media_player::{MediaItem, WatchPage};
//! # #[component]
//! # fn App() -> impl IntoView {
//! # let items = RwSignal::new(Vec::<MediaItem>::new());
//! # let edit_mode = RwSignal::new(false);
//! # let items_sig = Signal::derive(move || items.get());
//! view! {
//!     <WatchPage
//!         items=items_sig
//!         edit_mode=edit_mode
//!         audio=true
//!         artwork=Some("/covers/default.jpg".to_string())
//!     />
//! }
//! # }
//! ```
//!
//! ### Editable playlist
//!
//! Pass `on_rename`, `on_delete`, and `on_move` callbacks alongside an
//! `edit_mode` signal; when the signal is `true`, the playlist surfaces
//! per-row affordances. The callbacks are where you mutate your own state —
//! the component only reports the intent.
//!
//! ```rust,no_run
//! # use leptos::prelude::*;
//! # use leptos_media_player::{MediaItem, Uuid, WatchPage};
//! # #[component]
//! # fn App() -> impl IntoView {
//! let items = RwSignal::new(vec![
//!     MediaItem::new(Uuid::new_v4(), "One", "/1.mp4"),
//!     MediaItem::new(Uuid::new_v4(), "Two", "/2.mp4"),
//! ]);
//! let edit_mode = RwSignal::new(true);
//!
//! let on_rename = Callback::new(move |(id, new_title): (Uuid, String)| {
//!     items.update(|list| {
//!         if let Some(item) = list.iter_mut().find(|i| i.id == id) {
//!             item.title = new_title;
//!         }
//!     });
//! });
//! let on_delete = Callback::new(move |id: Uuid| {
//!     items.update(|list| list.retain(|i| i.id != id));
//! });
//! let on_move = Callback::new(move |(id, up): (Uuid, bool)| {
//!     items.update(|list| {
//!         if let Some(i) = list.iter().position(|x| x.id == id) {
//!             let j = if up {
//!                 i.saturating_sub(1)
//!             } else {
//!                 (i + 1).min(list.len().saturating_sub(1))
//!             };
//!             if i != j { list.swap(i, j); }
//!         }
//!     });
//! });
//!
//! let items_sig = Signal::derive(move || items.get());
//!
//! view! {
//!     <WatchPage
//!         items=items_sig
//!         edit_mode=edit_mode
//!         on_rename=on_rename
//!         on_delete=on_delete
//!         on_move=on_move
//!     />
//! }
//! # }
//! ```
//!
//! ### Lower level: you own the cursor
//!
//! When you need to observe or drive the playback cursor — persist it, sync
//! it to a URL query parameter, feed it to a third component — use
//! [`MediaPlayer`] and [`Playlist`] directly and pass them a shared signal.
//! Wrap them in your own flex container; the default `WatchPage` layout is
//! just one arrangement.
//!
//! ```rust,no_run
//! # use leptos::prelude::*;
//! # use leptos_media_player::{MediaItem, MediaPlayer, Playlist};
//! # #[component]
//! # fn App() -> impl IntoView {
//! # let items = RwSignal::new(Vec::<MediaItem>::new());
//! let current_idx = RwSignal::new(0usize);
//! let edit_mode   = RwSignal::new(false);
//!
//! // The cursor is ours to observe…
//! Effect::new(move |_| {
//!     let _i = current_idx.get();
//!     // e.g. persist, sync to URL, …
//! });
//!
//! let items_sig = Signal::derive(move || items.get());
//!
//! view! {
//!     <div class="flex flex-col gap-4 w-full lg:flex-row lg:items-start">
//!         <div class="flex-1 min-w-0 w-full lg:w-auto">
//!             <MediaPlayer items=items_sig current_idx=current_idx/>
//!         </div>
//!         <Playlist
//!             items=items_sig
//!             current_idx=current_idx
//!             edit_mode=edit_mode
//!         />
//!     </div>
//! }
//! # }
//! ```
//!
//! ## Wiring in a Leptos app
//!
//! Add the crate to `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! leptos_media_player = { path = "../leptos_media_player" }
//! ```
//!
//! …and drop [`WatchPage`] anywhere in your app's tree.

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

mod player;
mod playlist;
mod utils;
mod watch_page;

pub use player::MediaPlayer;
pub use playlist::Playlist;
pub use watch_page::WatchPage;

// Re-export `Uuid` so callers don't need to add `uuid` to their own
// `Cargo.toml` just to construct `MediaItem`s.
pub use uuid::Uuid;

/// One entry in the playlist.
///
/// `id` is the stable key used for reordering, renaming, and deletion; it
/// should be unique across a playlist. `title` is shown in both the player's
/// control overlay and the playlist row. `subtitle` is optional (a year,
/// duration, artist name, …). `src` is the URL handed to the `<video>`
/// element. `artwork` is an optional cover image URL used by the audio
/// layout; it overrides the component-level `artwork` fallback.
///
/// ```rust,no_run
/// # use leptos_media_player::{MediaItem, Uuid};
/// let item = MediaItem::new(Uuid::new_v4(), "Big Buck Bunny", "/media/bbb.mp4")
///     .with_subtitle("2008 · 9:56");
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MediaItem {
    /// Stable identifier; must be unique within a playlist.
    pub id: Uuid,
    /// Primary label shown in the controls and playlist.
    pub title: String,
    /// Optional secondary line shown under the title in the playlist.
    pub subtitle: Option<String>,
    /// Media URL (`mp4`, `webm`, `mp3`, stream, …) passed to `<video src>`.
    pub src: String,
    /// Optional cover image URL, used by the audio layout.
    pub artwork: Option<String>,
}

impl MediaItem {
    /// Build a minimal item with just an `id`, `title`, and `src`.
    ///
    /// ```rust,no_run
    /// # use leptos_media_player::{MediaItem, Uuid};
    /// let item = MediaItem::new(Uuid::new_v4(), "Intro", "/media/intro.mp4");
    /// ```
    pub fn new(id: Uuid, title: impl Into<String>, src: impl Into<String>) -> Self {
        Self {
            id,
            title: title.into(),
            subtitle: None,
            src: src.into(),
            artwork: None,
        }
    }

    /// Attach a subtitle line (builder style).
    ///
    /// ```rust,no_run
    /// # use leptos_media_player::{MediaItem, Uuid};
    /// let item = MediaItem::new(Uuid::new_v4(), "Intro", "/media/intro.mp4")
    ///     .with_subtitle("00:42");
    /// ```
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }
}

// ─── Context plumbing ─────────────────────────────────────────────────────
// Internal helper used to bundle several signals / callbacks into a single
// `provide_context` slot. Not part of the public API.

#[derive(Clone, Copy)]
struct Bundle<T>(T);

pub(crate) trait ContextBundle: Clone + Send + Sync + 'static {
    fn provide(self) {
        provide_context(Bundle(self));
    }
    fn expect() -> Self {
        expect_context::<Bundle<Self>>().0
    }
}
