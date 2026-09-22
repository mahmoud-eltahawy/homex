use crate::app::icons::{SearchIcon, XIcon};
use leptos::prelude::*;
use std::time::Duration;

// ─── Debounced search state machine ───────────────────────────────────────

#[derive(Clone, Copy)]
struct SearchHandlers {
    input_value: RwSignal<String>,
    on_input: Callback<web_sys::Event>,
    clear: Callback<()>,
}

fn use_search_debounce<F>(offset_reset: F, search_query: RwSignal<Option<String>>) -> SearchHandlers
where
    F: Fn() + Clone + Copy + Sync + Send + 'static,
{
    let input_value = RwSignal::new(search_query.get_untracked().unwrap_or_default());
    let debounce_handle: RwSignal<Option<TimeoutHandle>> = RwSignal::new(None);

    let clear = Callback::new(move |_: ()| {
        if let Some(handle) = debounce_handle.get_untracked() {
            handle.clear();
        }
        debounce_handle.set(None);

        batch(move || {
            input_value.set(String::new());
            search_query.set(None);
            offset_reset();
        });
    });

    let on_input = Callback::new(move |ev: web_sys::Event| {
        if let Some(handle) = debounce_handle.get_untracked() {
            handle.clear();
        }

        let text = event_target_value(&ev);
        input_value.set(text.clone());

        if text.is_empty() {
            debounce_handle.set(None);
            batch(move || {
                offset_reset();
                search_query.set(None);
            });
        } else {
            let text_clone = text.clone();
            let handle = set_timeout_with_handle(
                move || {
                    batch(move || {
                        offset_reset();
                        search_query.set(Some(text_clone));
                    });
                },
                Duration::from_millis(300),
            );
            debounce_handle.set(handle.ok());
        }
    });

    SearchHandlers {
        input_value,
        on_input,
        clear,
    }
}

// ─── View ─────────────────────────────────────────────────────────────────

#[component]
pub fn SearchBar<F>(offset_reset: F, search_query: RwSignal<Option<String>>) -> impl IntoView
where
    F: Fn() + Clone + Copy + Sync + Send + 'static,
{
    let handlers = use_search_debounce(offset_reset, search_query);

    view! {
        <div class="mb-2 md:mb-4 relative w-full max-w-md mx-auto">
            <span class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 pointer-events-none">
                <SearchIcon/>
            </span>

            <input
                autofocus=true
                type="text"
                class="w-full bg-gray-800 text-white rounded-xl pl-10 pr-10 py-3 outline-none \
                       focus:ring-2 focus:ring-cyan-400 transition-all"
                prop:value=move || handlers.input_value.get()
                on:input=move |ev| handlers.on_input.run(ev)
            />

            <Show when=move || !handlers.input_value.get().is_empty()>
                <button
                    on:click=move |_| handlers.clear.run(())
                    class="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 \
                           hover:text-white transition-colors"
                    aria-label="Clear search"
                >
                    <XIcon/>
                </button>
            </Show>
        </div>
    }
}
