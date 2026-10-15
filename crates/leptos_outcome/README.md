# leptos-outcome

Outcome-oriented `ResourceView` and `ErrorView` components for [Leptos](https://leptos.dev).

Most Leptos apps end up writing the same three things over and over:

1. A `match` around every `Resource<Result<T, E>>` for loading / error / success.
2. The same error card in every action's failure branch.
3. Some ad-hoc convention to tell a "retryable" error from a fatal one.

`leptos-outcome` packages all three into two components and one trait:

- **`ResourceView`** — renders `Resource<Result<T, E>>` as loading, success, or error.
- **`ErrorView`** — renders a `UserFacingError` in inline or banner layout.
- **`UserFacingError`** — the trait that drives both, decoupling the error type from the UI.

## Example

```rust,ignore
use leptos::prelude::*;
use leptos_outcome::{ErrorView, ResourceView, UserFacingError};

#[derive(Clone)]
struct AppError {
    message: String,
    is_fatal: bool,
}

impl UserFacingError for AppError {
    fn message(&self) -> String {
        self.message.clone()
    }

    fn is_fatal(&self) -> bool {
        self.is_fatal
    }
}

#[component]
fn Page() -> impl IntoView {
    let items = Resource::new(|| (), |_| fetch_items());
    view! {
        <ResourceView
            resource=items
            view_fn=ItemsList
            adapter=|v| v.into()
        />
    }
}
