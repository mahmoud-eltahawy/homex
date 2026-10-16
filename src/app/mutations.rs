//! The write side of the site's data contract.
//!
//! Reading a value is a [`Resource`] rendered by `ResourceView`. Writing one
//! is a [`Mutation`]: a server call that, on success, invalidates a fixed set
//! of `Resource`s and exposes `pending` / `error` signals for the UI to bind.
//!
//! - `Mutation::new(f)`       — closure taking the argument by reference
//! - `Mutation::for_1arg(f)`  — server fn taking one argument by value
//! - `Mutation::for_2args(f)` — server fn taking two arguments by value
//! - `.invalidates(r)`        — refetch `r` on success (chainable)
//! - `.on_success(f)`         — run `f(v)` when the mutation resolves Ok
//! - `.on_error(f)`           — run `f(e)` when the mutation resolves Err
//!
//! [`FieldEditor`] sits on top of a `Mutation` and owns the optimistic local
//! value. It is generic over two types:
//!
//! - `Raw` — what the editing widget emits (usually `String`)
//! - `T`   — what is actually stored (e.g. `Title`)
//!
//! `FieldEditor::new` has `Raw = T`; `FieldEditor::parsing` inserts a parse
//! step between the two.

use std::future::Future;

use leptos::prelude::*;
use leptos_outcome::action_error;
use serde::Serialize;
use serde::de::DeserializeOwned;

// ─── Mutation ────────────────────────────────────────────────────────────────

/// A dispatched server-side mutation.
///
/// `T` is the argument type; `V` is the success value (usually `()`);
/// `E` is the error type (usually `AppError`).
pub struct Mutation<T, V, E> {
    action: Action<T, Result<V, E>>,
}

impl<T, V, E> Copy for Mutation<T, V, E>
where
    T: Send + Sync + 'static,
    V: Send + Sync + 'static,
    E: Send + Sync + 'static,
{
}

impl<T, V, E> Clone for Mutation<T, V, E>
where
    T: Send + Sync + 'static,
    V: Send + Sync + 'static,
    E: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, V, E> Mutation<T, V, E>
where
    T: Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    E: Clone + Send + Sync + 'static,
{
    /// Build from a closure that takes its argument by reference.
    ///
    /// Prefer [`for_1arg`](Self::for_1arg) and
    /// [`for_2args`](Self::for_2args), which accept a server fn directly.
    pub fn new<F, Fut>(run: F) -> Self
    where
        F: Fn(&T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<V, E>> + Send + 'static,
    {
        Self {
            action: Action::new_local(move |t: &T| run(t)),
        }
    }

    /// Refetch `resource` when this mutation succeeds. Chainable.
    pub fn invalidates<R>(self, resource: Resource<R>) -> Self
    where
        R: Serialize + DeserializeOwned + Send + Sync + Clone + 'static,
    {
        let action = self.action;
        Effect::new(move |_| {
            if action.value().get().is_some_and(|r| r.is_ok()) {
                resource.refetch();
            }
        });
        self
    }

    /// Run `f(v)` when this mutation resolves to `Ok(v)`. Chainable.
    pub fn on_success<F>(self, f: F) -> Self
    where
        F: Fn(V) + Send + Sync + 'static,
    {
        let action = self.action;
        Effect::new(move |_| {
            if let Some(Ok(v)) = action.value().get() {
                f(v);
            }
        });
        self
    }

    /// Run `f(e)` when this mutation resolves to `Err(e)`. Chainable.
    pub fn on_error<F>(self, f: F) -> Self
    where
        F: Fn(E) + Send + Sync + 'static,
    {
        let action = self.action;
        Effect::new(move |_| {
            if let Some(Err(e)) = action.value().get() {
                f(e);
            }
        });
        self
    }

    /// Dispatch with a value.
    pub fn dispatch(&self, t: T) {
        self.action.dispatch(t);
    }

    /// Whether a request is in flight.
    pub fn pending(&self) -> Signal<bool> {
        self.action.pending().into()
    }

    /// The last error, if the last run failed.
    pub fn error(&self) -> Signal<Option<E>> {
        action_error(self.action)
    }

    /// The last result, if the mutation has run.
    pub fn value(&self) -> Signal<Option<Result<V, E>>> {
        self.action.value().into()
    }
}

// ─── Tuple-specialised constructors ──────────────────────────────────────────

impl<A, V, E> Mutation<A, V, E>
where
    A: Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    E: Clone + Send + Sync + 'static,
{
    /// Build from a server fn taking one argument by value.
    pub fn for_1arg<F, Fut>(run: F) -> Self
    where
        F: Fn(A) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<V, E>> + Send + 'static,
    {
        Self::new(move |a: &A| run(a.clone()))
    }
}

impl<A, B, V, E> Mutation<(A, B), V, E>
where
    A: Clone + Send + Sync + 'static,
    B: Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
    E: Clone + Send + Sync + 'static,
{
    /// Build from a server fn taking two arguments by value.
    pub fn for_2args<F, Fut>(run: F) -> Self
    where
        F: Fn(A, B) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<V, E>> + Send + 'static,
    {
        Self::new(move |(a, b): &(A, B)| run(a.clone(), b.clone()))
    }

    /// A `Callback<B>` that dispatches with a fixed first argument.
    pub fn callback_with_id(&self, id: A) -> Callback<B> {
        let action = self.action;
        Callback::new(move |b: B| {
            action.dispatch((id.clone(), b));
        })
    }
}

// ─── FieldEditor ─────────────────────────────────────────────────────────────

/// An editable field backed by a [`Mutation`].
///
/// `Raw` is what the editing widget emits (usually `String`). `T` is what is
/// stored. `E` is the mutation's error type.
pub struct FieldEditor<Raw, T, E>
where
    Raw: Send + Sync + 'static,
    T: Send + Sync + 'static,
    E: Send + Sync + 'static,
{
    value: RwSignal<T>,
    on_commit: Callback<Raw>,
    error: Signal<Option<E>>,
}

impl<Raw, T, E> Copy for FieldEditor<Raw, T, E>
where
    Raw: Send + Sync + 'static,
    T: Send + Sync + 'static,
    E: Send + Sync + 'static,
{
}

impl<Raw, T, E> Clone for FieldEditor<Raw, T, E>
where
    Raw: Send + Sync + 'static,
    T: Send + Sync + 'static,
    E: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<T, E> FieldEditor<T, T, E>
where
    T: Clone + Send + Sync + 'static,
    E: Clone + Send + Sync + 'static,
{
    /// Build a field whose editor and store share a type.
    pub fn new<Id, V>(initial: T, id: Id, mutation: Mutation<(Id, T), V, E>) -> Self
    where
        Id: Clone + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
    {
        let value = RwSignal::new(initial);
        let on_commit = Callback::new(move |t: T| {
            value.set(t.clone());
            mutation.dispatch((id.clone(), t));
        });
        Self {
            value,
            on_commit,
            error: mutation.error(),
        }
    }
}

impl<Raw, T, E> FieldEditor<Raw, T, E>
where
    Raw: Send + Sync + 'static,
    T: Clone + Send + Sync + 'static,
    E: Clone + Send + Sync + 'static,
{
    /// Build a field whose editor emits `Raw` and parses it into `T`.
    pub fn parsing<F, Id, V>(
        initial: T,
        id: Id,
        parse: F,
        mutation: Mutation<(Id, T), V, E>,
    ) -> Self
    where
        F: Fn(Raw) -> Option<T> + Send + Sync + 'static,
        Id: Clone + Send + Sync + 'static,
        V: Clone + Send + Sync + 'static,
    {
        let value = RwSignal::new(initial);
        let on_commit = Callback::new(move |raw: Raw| {
            let Some(t) = parse(raw) else {
                return;
            };
            value.set(t.clone());
            mutation.dispatch((id.clone(), t));
        });
        Self {
            value,
            on_commit,
            error: mutation.error(),
        }
    }

    pub fn value(&self) -> RwSignal<T> {
        self.value
    }

    pub fn on_commit(&self) -> Callback<Raw> {
        self.on_commit
    }

    pub fn error(&self) -> Signal<Option<E>> {
        self.error
    }

    /// The value as `Signal<String>`, ready for `EditableText::value`.
    pub fn text(&self) -> Signal<String>
    where
        T: std::fmt::Display,
    {
        let value = self.value;
        Signal::derive(move || value.get().to_string())
    }
}
