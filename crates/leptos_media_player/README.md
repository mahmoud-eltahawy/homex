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

- Styled entirely with Tailwind utility classes — no external CSS file, no
  custom-property theming layer.
- Responsive: stacks on phones and tablet portrait, side-by-side from
  `lg` (`≥ 1024 px`) up.
- Composable: `MediaPlayer` and `Playlist` are independent primitives that
  share a cursor signal; `WatchPage` is a convenience composition that owns
  it for you.

