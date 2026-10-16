use crate::app::icons::{SearchIcon, XIcon};
use crate::app::types::TagName;
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

// ─── Tag hint chip ────────────────────────────────────────────────────────

fn extract_tag_hints(raw: &str) -> Vec<TagName> {
    let mut tags: Vec<TagName> = raw
        .split_whitespace()
        .filter_map(|w| w.strip_prefix('#'))
        .filter_map(|t| TagName::new(t).ok())
        .collect();
    tags.sort();
    tags.dedup();
    tags
}

#[component]
fn TagHints(raw: Signal<String>) -> impl IntoView {
    let hints = move || extract_tag_hints(&raw.get());

    view! {
        <Show when=move || !hints().is_empty()>
            <div class="mt-2 flex flex-wrap items-center gap-1.5 justify-center">
                <span class="text-[10px] text-gray-500 uppercase tracking-wider">
                    "filtering by tag"
                </span>
                <For each=hints key=|t| t.clone() let:tag>
                    <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full \
                                 bg-cyan-500/15 border border-cyan-400/30 text-cyan-300 \
                                 text-[10px] font-medium">
                        {"#"}{tag.to_string()}
                    </span>
                </For>
            </div>
        </Show>
    }
}

// ─── View ─────────────────────────────────────────────────────────────────

#[component]
pub fn SearchBar<F>(offset_reset: F, search_query: RwSignal<Option<String>>) -> impl IntoView
where
    F: Fn() + Clone + Copy + Sync + Send + 'static,
{
    let handlers = use_search_debounce(offset_reset, search_query);

    let raw = Signal::derive(move || handlers.input_value.get());

    view! {
        <div class="mb-2 md:mb-4 w-full max-w-md mx-auto">
            <div class="relative w-full">
                <span class="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400 pointer-events-none">
                    <SearchIcon/>
                </span>

                <input
                    autofocus=true
                    type="text"
                    placeholder="Search... use #tag to filter"
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
            <TagHints raw=raw/>
        </div>
    }
}
