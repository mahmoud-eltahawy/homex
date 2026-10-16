# leptos_page

Page-as-a-value routing for [Leptos](https://leptos.dev).

Most Leptos apps end up with a `<Route>` per page, and every page repeats the
same three things:

1. Parse the URL parameters, and decide what to do when they don't parse.
2. Build the resources that depend on those parameters.
3. Wire the loading / error / success branches.

`leptos_page` packages this into one trait and one type:

- **`PageContent<E>`** — how a page loads and renders itself.
- **`Page<T, E>`** — the routing wrapper the router actually sees.
- **`RouteError`** — the default `E`, produced by every parameter reader.

