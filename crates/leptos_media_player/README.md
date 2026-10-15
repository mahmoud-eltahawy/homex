# leptos_media_player

> ⚠️ **This crate is vibe coded.**
>
> It was written with heavy AI assistance and has **not** been battle-tested
> in production. There are no unit or integration tests for the hotkey
> dispatch, the DOM bindings, or the edge cases around seeking, fullscreen,
> and playlist mutation — only doctests on the public API. Treat it as a
> starting point, not a dependency you can rely on without reading the
> source. If you need something battle-tested, use
> [Video.js](https://videojs.com), [Plyr](https://plyr.io), or
> [Vidstack](https://www.vidstack.io) — or wrap one of those with a thin
> Leptos binding.

A media player component for [Leptos](https://leptos.dev), with a
YouTube-style watch page, a scrollable playlist column, an optional
edit-mode for renaming / deleting / reordering, and keyboard shortcuts.

- One embedded stylesheet, injected automatically — no CSS file to wire up.
- Responsive: stacks on phones and tablet portrait, side-by-side from
  1024 px up.
- Composable: `MediaPlayer` and `Playlist` are independent primitives that
  share a cursor signal; `WatchPage` is a convenience composition that owns
  it for you.
- Themed via CSS custom properties.

---

## Install

```toml
[dependencies]
leptos_media_player = { git = "https://github.com/mahmoud_eltahawy/leptos_media_player.git" }
# or, once published:
# leptos_media_player = "0.1"
```

Requires Leptos `0.8`.

---

## Quickstart

Drop `WatchPage` into any Leptos component:

```rust
use leptos::prelude::*;
use leptos_media_player::{MediaItem, WatchPage};

#[component]
pub fn App() -> impl IntoView {
    let items = RwSignal::new(vec![
        MediaItem::new(1, "Big Buck Bunny", "/media/bbb.mp4")
            .with_subtitle("2008 · 9:56"),
        MediaItem::new(2, "Sintel", "/media/sintel.mp4")
            .with_subtitle("2010 · 14:48"),
    ]);
    let edit_mode = RwSignal::new(false);
    let items_sig = Signal::derive(move || items.get());

    view! {
        <WatchPage items=items_sig edit_mode=edit_mode />
    }
}
```

`WatchPage` clamps the cursor, hides the playlist when there's only one
item, and stacks or splits the layout responsively. No other setup.

---

## What's in the box

| Item            | What it does                                                                                                                  |
|-----------------|-------------------------------------------------------------------------------------------------------------------------------|
| `MediaItem`     | The playlist entry type: `id`, `title`, optional `subtitle`, `src`, optional `artwork`.                                        |
| `MediaPlayer`   | A `<video>` (or audio) surface with play/pause, seek, volume, mute, fullscreen, and prev/next controls.                        |
| `Playlist`      | A scrollable list of `MediaItem`s with optional edit-mode (rename / delete / reorder) and download links.                      |
| `WatchPage`     | A convenience wrapper that lays the two out YouTube-style: player on the left, playlist on the right. Stacks vertically on phones. |

---

## API reference

### `MediaItem`

The playlist entry type. `Clone + Debug + PartialEq + Serialize + Deserialize`.

| Field      | Type              | Description                                                                                       |
|------------|-------------------|---------------------------------------------------------------------------------------------------|
| `id`       | `u64`             | Stable identifier. Must be unique within a playlist — used as the `For` key, and as the argument to `on_rename` / `on_delete` / `on_move`. |
| `title`    | `String`          | Primary label. Shown in the controls overlay and the playlist row.                                |
| `subtitle` | `Option<String>`  | Optional secondary line under the title in the playlist.                                          |
| `src`      | `String`          | Media URL (`mp4`, `webm`, `mp3`, stream, …) passed to the `<video src>` attribute.                |
| `artwork`  | `Option<String>`  | Optional cover image URL. Only used by the audio layout; overrides the component-level fallback.  |

Constructors:

```rust
// Minimal:
let item = MediaItem::new(1, "Intro", "/media/intro.mp4");

// With a subtitle (builder style):
let item = MediaItem::new(1, "Intro", "/media/intro.mp4")
    .with_subtitle("00:42");
```

All fields are `pub`, so you can also construct with struct literal syntax:

```rust
let item = MediaItem {
    id: 1,
    title: "Intro".into(),
    subtitle: Some("00:42".into()),
    src: "/media/intro.mp4".into(),
    artwork: None,
};
```

---

### `MediaPlayer`

The video (or audio) surface with its controls.

**Pair it with `Playlist`** sharing the same `items` and `current_idx`
signals, or use it standalone.

#### Props

| Prop          | Type                   | Default   | Notes                                                                                                      |
|---------------|------------------------|-----------|-------------------------------------------------------------------------------------------------------------|
| `items`       | `Signal<Vec<MediaItem>>` | required  | The playlist. The component derives the current source, title, and artwork from this.                       |
| `current_idx` | `RwSignal<usize>`        | required  | The cursor. Shared with a sibling `Playlist` if you compose them yourself.                                  |
| `audio`       | `bool`                   | `false`   | When `true`, hides the video surface and renders an artwork overlay instead.                                |
| `artwork`     | `Option<String>`         | `None`    | Default cover image for the audio layout. Per-item `MediaItem::artwork` wins when set.                      |

#### Example

```rust
let current_idx = RwSignal::new(0usize);
let items_sig = Signal::derive(move || items.get());

view! {
    <MediaPlayer items=items_sig current_idx=current_idx/>
}
```

#### Notes

- The component is focusable (`tabindex="0"`) so hotkeys work once the user
  clicks or tabs into it.
- Fullscreen targets the player **container**, not the `<video>` element,
  so the controls overlay stays visible in fullscreen.
- An internal clamp effect keeps `current_idx` in range when `items` shrinks
  (e.g. after a delete).

---

### `Playlist`

A scrollable sidebar listing every item. Renders an `<aside>` sized for a
right-hand column; the caller decides whether to show it.

#### Props

| Prop            | Type                                    | Default        | Notes                                                                                                |
|-----------------|-----------------------------------------|----------------|--------------------------------------------------------------------------------------------------------|
| `items`         | `Signal<Vec<MediaItem>>`                | required       | The playlist.                                                                                          |
| `current_idx`   | `RwSignal<usize>`                       | required       | The cursor. Shared with a sibling `MediaPlayer`.                                                       |
| `edit_mode`     | `Signal<bool>` (`#[prop(into)]`)        | required       | When `true`, per-row rename / delete / move affordances appear.                                        |
| `title`         | `Option<String>`                        | `None` → `"Playlist"` | Header text.                                                                                    |
| `on_rename`     | `Option<Callback<(u64, String)>>`       | `None`         | Called with `(id, new_title)` on commit. Enables the rename button in edit mode.                       |
| `on_delete`     | `Option<Callback<u64>>`                 | `None`         | Called with `id`. Enables the delete button in edit mode.                                              |
| `on_move`       | `Option<Callback<(u64, bool)>>`         | `None`         | Called with `(id, is_up)`. Enables the reorder arrows in edit mode.                                    |
| `show_download` | `bool`                                  | `true`         | Shows a per-row download link pointing at `MediaItem::src`.                                            |

#### Example

```rust
let on_rename = Callback::new(move |(id, new_title): (u64, String)| {
    items.update(|list| {
        if let Some(item) = list.iter_mut().find(|i| i.id == id) {
            item.title = new_title;
        }
    });
});
let on_delete = Callback::new(move |id: u64| {
    items.update(|list| list.retain(|i| i.id != id));
});
let on_move = Callback::new(move |(id, up): (u64, bool)| {
    items.update(|list| {
        if let Some(i) = list.iter().position(|x| x.id == id) {
            let j = if up {
                i.saturating_sub(1)
            } else {
                (i + 1).min(list.len().saturating_sub(1))
            };
            if i != j { list.swap(i, j); }
        }
    });
});

view! {
    <Playlist
        items=items_sig
        current_idx=current_idx
        edit_mode=edit_mode
        on_rename=on_rename
        on_delete=on_delete
        on_move=on_move
    />
}
```

#### Notes

- Each callback is optional. Passing only some of them enables only the
  corresponding affordances.
- Rename commits on `Enter` or blur; `Escape` cancels.
- Move arrows are disabled at the first and last row respectively.

---

### `WatchPage`

Player + playlist in the default YouTube-style layout. **Owns** its cursor —
you pass an initial index, it creates the signal internally.

#### Props

| Prop             | Type                                    | Default          | Notes                                                                                        |
|------------------|-----------------------------------------|------------------|----------------------------------------------------------------------------------------------|
| `items`          | `Signal<Vec<MediaItem>>`                | required         | The playlist.                                                                                |
| `initial_idx`    | `usize`                                 | `0`              | Cursor on mount. Clamped to `items.len() - 1`; treated as `0` when `items` is empty.          |
| `edit_mode`      | `Signal<bool>` (`#[prop(into)]`)        | required         | Forwarded to `Playlist`.                                                                     |
| `audio`          | `bool`                                  | `false`          | Forwarded to `MediaPlayer`.                                                                  |
| `artwork`        | `Option<String>`                        | `None`           | Forwarded to `MediaPlayer`.                                                                  |
| `playlist_title` | `Option<String>`                        | `None`           | Forwarded to `Playlist` as `title`.                                                          |
| `on_rename`      | `Option<Callback<(u64, String)>>`       | `None`           | Forwarded to `Playlist`.                                                                     |
| `on_delete`      | `Option<Callback<u64>>`                 | `None`           | Forwarded to `Playlist`.                                                                     |
| `on_move`        | `Option<Callback<(u64, bool)>>`         | `None`           | Forwarded to `Playlist`.                                                                     |
| `show_download`  | `bool`                                  | `true`           | Forwarded to `Playlist`.                                                                     |

#### Notes

- The playlist is hidden automatically when `items.len() <= 1`.
- `initial_idx` is **initial only**. Changing it after mount has no effect,
  and the cursor is not exposed upward. If you need to observe or drive the
  cursor, use `MediaPlayer` + `Playlist` directly with your own signal.

---

## Composition: `WatchPage` vs the primitives

`MediaPlayer` and `Playlist` are **primitives**. They take the cursor as a
shared `RwSignal<usize>` and know nothing about each other. You can place
them side by side, one above the other, inside a modal, share one playlist
between two players, wire the cursor to a URL query parameter, persist it
to `localStorage`, and so on.

`WatchPage` is a **convenience composition**. It owns the cursor, lays the
two out in the default arrangement, and hides the playlist when there's
nothing to show.

Reach for `WatchPage` for the common case. Drop to the primitives when you
need to own or observe the cursor, or when the layout isn't the default:

```rust
let current_idx = RwSignal::new(0usize);
let edit_mode   = RwSignal::new(false);

// Observe the cursor:
Effect::new(move |_| {
    let _i = current_idx.get();
    // e.g. persist, sync to URL, …
});

let items_sig = Signal::derive(move || items.get());

view! {
    {stylesheet()}
    <div class="lmp-watch">
        <div class="lmp-watch-main">
            <MediaPlayer items=items_sig current_idx=current_idx/>
        </div>
        <Playlist
            items=items_sig
            current_idx=current_idx
            edit_mode=edit_mode
        />
    </div>
}
```

The `{stylesheet()}` call at the top of the parent's view is optional when
you use the primitives directly. It ensures only one `<style>` tag is
emitted for the whole subtree — without it, `MediaPlayer` and `Playlist`
would each inject their own copy of the (identical) stylesheet. Harmless,
but wasteful. `WatchPage` does this for you.

---

## Keyboard shortcuts

Click anywhere on the player (or Tab into it) to focus it, then:

| Key            | Action                    |
|----------------|---------------------------|
| `Space`, `k`   | Toggle play / pause       |
| `m`            | Toggle mute               |
| `f`            | Toggle fullscreen         |
| `←` / `→`      | Seek back / forward 5 s   |
| `↑` / `↓`      | Volume up / down 5 %      |
| `0` – `9`      | Jump to 0 % – 90 %        |
| `Home` / `End` | Jump to start / end       |

Shortcuts are scoped to the player:

- They never fire page-wide — only when the player or one of its children
  has focus.
- They stay out of the way while you type in a form field (rename input,
  any page-level input, contenteditable) or nudge a slider.
- Browser/OS shortcuts (`Ctrl`/`Cmd`/`Alt` + key) pass through untouched.

---

## Theming

All styles live in the crate's embedded stylesheet, injected automatically
by the components. No external CSS file to import. Theme by overriding the
CSS custom properties on `:root` or any ancestor:

```css
:root {
    --lmp-accent: #f472b6;
    --lmp-radius: 0.75rem;
}
```

### All custom properties

| Variable                | Default                                   | Purpose                                    |
|-------------------------|-------------------------------------------|--------------------------------------------|
| `--lmp-accent`          | `#22d3ee`                                 | Primary accent (slider thumbs, current row). |
| `--lmp-accent-bg`       | `rgba(6, 182, 212, 0.15)`                 | Accent background tint.                    |
| `--lmp-accent-border`   | `rgba(6, 182, 212, 0.3)`                  | Accent border tint.                        |
| `--lmp-accent-glow`     | `rgba(34, 211, 238, 0.3)`                 | Slider thumb glow shadow.                  |
| `--lmp-surface`         | `rgba(255, 255, 255, 0.05)`               | Panel and row hover surface.               |
| `--lmp-surface-hover`   | `rgba(255, 255, 255, 0.1)`                | Button and input hover surface.            |
| `--lmp-border`          | `rgba(255, 255, 255, 0.1)`                | Panel and divider borders.                 |
| `--lmp-text`            | `#ffffff`                                 | Primary text.                              |
| `--lmp-text-dim`        | `#9ca3af`                                 | Secondary text (subtitles, counts).        |
| `--lmp-text-muted`      | `#6b7280`                                 | Muted text (row arrows).                   |
| `--lmp-danger`          | `#fca5a5`                                 | Delete-action hover color.                 |
| `--lmp-player-bg`       | `#000`                                    | Video surface background.                  |
| `--lmp-audio-bg`        | `linear-gradient(to bottom right, …)`     | Audio artwork background.                  |
| `--lmp-radius`          | `1rem`                                    | Panel and player corner radius.            |
| `--lmp-radius-sm`       | `0.5rem`                                  | Row corner radius.                         |
| `--lmp-radius-xs`       | `0.25rem`                                 | Inline input corner radius.                |

### Responsive breakpoints

| Width          | Layout                                                                       |
|----------------|------------------------------------------------------------------------------|
| `< 480 px`     | Volume slider hidden; mute button alone.                                     |
| `≥ 640 px`     | Larger controls padding; taller video cap (60 vh).                           |
| `≥ 1024 px`    | Watch page side-by-side; playlist column at `18rem`; video cap at 75 vh.     |
| `≥ 1280 px`    | Playlist column at `20rem`.                                                  |
| `≥ 1536 px`    | Playlist column at `24rem`.                                                  |

`(pointer: coarse)` devices get larger tap targets. `prefers-reduced-motion`
disables transitions.

---

## Limitations

Deliberately not included in this crate, but worth knowing about before you
decide it fits your use case:

- **No HLS or DASH.** Only formats the browser's native `<video>` element
  can decode: MP4, WebM, Ogg. Adaptive streaming would require integrating
  `hls.js` or `dash.js` via web-sys.
- **No captions or subtitles.** No `<track>` support, no captions button.
  This is an accessibility gap.
- **No picture-in-picture.** `video.requestPictureInPicture()` is not
  wired up.
- **No playback-rate control.** Speed is locked at 1×.
- **No plugin system.** Features are baked into the components.
- **No analytics, casting, chapters, or thumbnails.**

See [Video.js](https://videojs.com) for a feature-rich alternative if any
of the above are requirements.

---

## License

Licensed under either of

- Apache License, Version 2.0 ([`LICENSE-APACHE`](LICENSE-APACHE) or
  <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([`LICENSE-MIT`](LICENSE-MIT) or
  <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
