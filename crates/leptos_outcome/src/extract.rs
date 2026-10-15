//! Signal adapters that pull the error out of an [`Action`](leptos::prelude::Action),
//! a [`Resource`](leptos::prelude::Resource), or a hand-rolled
//! `Signal<Option<Result<T, E>>>`.
//!
//! Every function here returns `Signal<Option<E>>`, ready to bind to
//! [`ErrorView`](crate::ErrorView).

use leptos::prelude::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Pull the error out of an [`Action<_, Result<V, E>>`](leptos::prelude::Action).
///
/// Returns a signal that is `Some(e)` whenever the action's last result was
/// `Err(e)`, and `None` otherwise. Bind it directly to
/// [`ErrorView`](crate::ErrorView).
///
/// # Example
///
/// ```rust,ignore
/// let submit = Action::new(|t: &String| create(t.clone()));
/// view! { <ErrorView error=action_error(submit)/> }
/// ```
pub fn action_error<A, V, E>(action: Action<A, Result<V, E>>) -> Signal<Option<E>>
where
    A: Send + Sync + 'static,
    V: Send + Sync + Clone + 'static,
    E: Send + Sync + Clone + 'static,
{
    Signal::derive(move || action.value().get().and_then(|r| r.err()))
}

/// Pull the error out of a [`Resource<Result<T, E>>`](leptos::prelude::Resource).
///
/// Returns `Some(e)` when the resource has resolved to `Err(e)`, and `None`
/// while it is still loading or has resolved successfully.
pub fn resource_error<T, E>(resource: Resource<Result<T, E>>) -> Signal<Option<E>>
where
    T: Serialize + DeserializeOwned + Send + Sync + Clone + 'static,
    E: Send + Sync + Clone + 'static,
{
    Signal::derive(move || resource.get().and_then(|r| r.err()))
}

/// Pull the error out of a hand-rolled `Signal<Option<Result<T, E>>>`.
///
/// Useful when the error source isn't a Leptos `Action` or `Resource` —
/// for instance when it's threaded through your own state machine.
pub fn result_signal_error<T, E>(result: Signal<Option<Result<T, E>>>) -> Signal<Option<E>>
where
    T: Send + Sync + Clone + 'static,
    E: Send + Sync + Clone + 'static,
{
    Signal::derive(move || result.get().and_then(|r| r.err()))
}
