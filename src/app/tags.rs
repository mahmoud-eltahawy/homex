use leptos::prelude::*;

use crate::app::inline_edit::use_edit_mode;
use crate::app::model::Tag;
use uuid::Uuid;

#[cfg(feature = "ssr")]
use crate::app::server::{ToastyErr, db};

// ─── Tag pill ─────────────────────────────────────────────────────────────

#[component]
pub fn TagPill(#[prop(into)] name: String) -> impl IntoView {
    view! {
        <span class="inline-flex items-center px-2 py-0.5 rounded-full \
                     bg-cyan-500/15 border border-cyan-400/25 text-cyan-300 \
                     text-[10px] font-medium tracking-wide whitespace-nowrap">
            {name}
        </span>
    }
}

// ─── Tag list (read-only, truncating) ─────────────────────────────────────

#[component]
pub fn TagList(tags: Vec<Tag>, #[prop(optional)] max: Option<usize>) -> impl IntoView {
    let total = tags.len();
    let cap = max.unwrap_or(usize::MAX);
    let overflow = total.saturating_sub(cap);
    let shown: Vec<Tag> = tags.into_iter().take(cap).collect();

    let pills = shown
        .into_iter()
        .map(|t| view! { <TagPill name=t.name/> })
        .collect_view();

    let overflow_chip = (overflow > 0).then(|| {
        view! {
            <span class="text-[10px] text-gray-500 font-mono whitespace-nowrap">
                {"+"}{overflow}
            </span>
        }
    });

    view! {
        {if total == 0 {
            None
        } else {
            Some(view! {
                <div class="flex flex-wrap items-center gap-1.5">
                    {pills}
                    {overflow_chip}
                </div>
            })
        }}
    }
}

// ─── Server fn ────────────────────────────────────────────────────────────

#[server]
pub async fn fetch_all_tags() -> Result<Vec<Tag>, ServerFnError> {
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    db::fetch_all_tags(&mut state.db)
        .await
        .srv_ctx("fetch all tags")
}

// ─── Shared tag cache ─────────────────────────────────────────────────────

#[derive(Clone, Copy)]
pub struct TagsCache {
    all: Resource<Result<Vec<Tag>, ServerFnError>>,
    tick: RwSignal<u32>,
}

impl TagsCache {
    pub fn new() -> Self {
        let tick = RwSignal::new(0u32);
        let all = Resource::new(move || tick.get(), |_| fetch_all_tags());
        Self { all, tick }
    }

    pub fn provide(self) {
        provide_context(self);
    }

    pub fn refresh(&self) {
        self.tick.update(|n| *n = n.wrapping_add(1));
    }
}

pub fn use_tags_cache() -> TagsCache {
    use_context::<TagsCache>().unwrap_or_else(TagsCache::new)
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn ci_eq(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

fn ci_contains(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

// ─── Tag editor with autocomplete ─────────────────────────────────────────

#[component]
pub fn TagEditor(tags: RwSignal<Vec<Tag>>, on_commit: Callback<Vec<String>>) -> impl IntoView {
    let edit_mode = use_edit_mode();
    let cache = use_tags_cache();
    let input = RwSignal::new(String::new());
    let focused = RwSignal::new(false);

    let all_names: Memo<Vec<String>> = Memo::new(move |_| {
        cache
            .all
            .get()
            .and_then(|r| r.ok())
            .unwrap_or_default()
            .into_iter()
            .map(|t| t.name)
            .collect()
    });

    let set_names = Callback::new(move |names: Vec<String>| {
        let new_tags: Vec<Tag> = names
            .iter()
            .map(|n| Tag {
                id: Uuid::nil(),
                name: n.clone(),
            })
            .collect();
        tags.set(new_tags);
        on_commit.run(names);
        cache.refresh();
    });

    let add_name = Callback::new(move |name: String| {
        let trimmed = name.trim().to_string();
        input.set(String::new());
        if trimmed.is_empty() {
            return;
        }
        let mut names: Vec<String> = tags.get_untracked().into_iter().map(|t| t.name).collect();
        if names.iter().any(|n| ci_eq(n, &trimmed)) {
            return;
        }
        names.push(trimmed);
        names.sort();
        set_names.run(names);
    });

    let remove = Callback::new(move |name: String| {
        let names: Vec<String> = tags
            .get_untracked()
            .into_iter()
            .map(|t| t.name)
            .filter(|x| x != &name)
            .collect();
        set_names.run(names);
    });

    let on_keydown = Callback::new(move |ev: web_sys::KeyboardEvent| match ev.key().as_str() {
        "Enter" => {
            ev.prevent_default();
            let t = input.get_untracked();
            if !t.trim().is_empty() {
                add_name.run(t);
            }
        }
        "Backspace" if input.get_untracked().is_empty() => {
            let mut names: Vec<String> = tags.get_untracked().into_iter().map(|t| t.name).collect();
            if names.pop().is_some() {
                set_names.run(names);
            }
        }
        "Escape" => input.set(String::new()),
        _ => {}
    });

    let suggestions: Memo<Vec<String>> = Memo::new(move |_| {
        let q = input.get().trim().to_string();
        if q.is_empty() {
            return Vec::new();
        }
        let added: Vec<String> = tags.get().into_iter().map(|t| t.name).collect();
        all_names
            .get()
            .into_iter()
            .filter(|n| ci_contains(n, &q))
            .filter(|n| !added.iter().any(|a| ci_eq(a, n)))
            .take(6)
            .collect()
    });

    let exact_existing: Memo<bool> = Memo::new(move |_| {
        let q = input.get().trim().to_string();
        if q.is_empty() {
            return false;
        }
        let added: Vec<String> = tags.get().into_iter().map(|t| t.name).collect();
        if added.iter().any(|a| ci_eq(a, &q)) {
            return false;
        }
        all_names.get().iter().any(|n| ci_eq(n, &q))
    });

    let show_create: Memo<bool> = Memo::new(move |_| {
        let q = input.get().trim().to_string();
        if q.is_empty() {
            return false;
        }
        let added: Vec<String> = tags.get().into_iter().map(|t| t.name).collect();
        !added.iter().any(|a| ci_eq(a, &q)) && !all_names.get().iter().any(|n| ci_eq(n, &q))
    });

    let already_added: Memo<bool> = Memo::new(move |_| {
        let q = input.get().trim().to_string();
        if q.is_empty() {
            return false;
        }
        tags.get().iter().any(|t| ci_eq(&t.name, &q))
    });

    let render = move || {
        if !edit_mode.get() {
            let current = tags.get();
            if current.is_empty() {
                return view! {
                    <span class="text-[10px] text-gray-500 italic">"no tags"</span>
                }
                .into_any();
            }
            return view! {
                <div class="flex flex-wrap items-center gap-1.5">
                    <For each=move || tags.get() key=|t| t.name.clone() let:tag>
                        <TagPill name=tag.name/>
                    </For>
                </div>
            }
            .into_any();
        }

        view! {
            <div class="flex flex-wrap items-center gap-1.5">
                <For each=move || tags.get() key=|t| t.name.clone() let:tag>
                    {
                        let name = tag.name.clone();
                        let name_for_remove = name.clone();
                        view! {
                            <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full \
                                         bg-cyan-500/15 border border-cyan-400/25 text-cyan-300 \
                                         text-[10px] font-medium tracking-wide">
                                {name}
                                <button
                                    type="button"
                                    on:click=move |_| remove.run(name_for_remove.clone())
                                    class="text-cyan-300/70 hover:text-white transition-colors leading-none"
                                    aria-label="Remove tag"
                                >
                                    "×"
                                </button>
                            </span>
                        }
                    }
                </For>

                <div class="relative">
                    <input
                        type="text"
                        placeholder="add tag..."
                        prop:value=move || input.get()
                        on:input=move |ev| input.set(event_target_value(&ev))
                        on:keydown=move |ev| on_keydown.run(ev)
                        on:focus=move |_| focused.set(true)
                        on:blur=move |_| focused.set(false)
                        class="bg-white/5 border border-white/10 rounded-full px-2 py-0.5 \
                               text-[10px] text-white placeholder:text-gray-500 \
                               focus:outline-none focus:border-cyan-400/40 w-28"
                    />

                    <Show when=move || focused.get() && !input.get().trim().is_empty()>
                        <div class="absolute top-full left-0 mt-1 min-w-[180px] max-w-[260px] z-30 \
                                    bg-[#12121a] border border-white/10 rounded-lg shadow-2xl \
                                    py-1 max-h-48 overflow-auto">
                            <For each=move || suggestions.get() key=|s| s.clone() let:name>
                                {
                                    let name_for_add = name.clone();
                                    view! {
                                        <button
                                            type="button"
                                            on:mousedown=move |ev| {
                                                ev.prevent_default();
                                                add_name.run(name_for_add.clone());
                                            }
                                            class="w-full text-left px-2 py-1 flex items-center gap-2 \
                                                   hover:bg-white/10 transition-colors"
                                        >
                                            <span class="text-cyan-400 text-[10px] leading-none">"✓"</span>
                                            <span class="text-white text-xs truncate flex-1">
                                                {name.clone()}
                                            </span>
                                            <span class="text-[9px] text-gray-500 ml-auto shrink-0">
                                                "existing"
                                            </span>
                                        </button>
                                    }
                                }
                            </For>

                            <Show when=move || exact_existing.get()>
                                <div class="px-2 py-1 flex items-center gap-2 border-t border-white/5">
                                    <span class="text-cyan-400 text-[10px] leading-none">"✓"</span>
                                    <span class="text-gray-400 text-[10px] italic">
                                        "tag already exists — press Enter to attach"
                                    </span>
                                </div>
                            </Show>

                            <Show when=move || show_create.get()>
                                <button
                                    type="button"
                                    on:mousedown=move |ev| {
                                        ev.prevent_default();
                                        let q = input.get_untracked();
                                        add_name.run(q);
                                    }
                                    class="w-full text-left px-2 py-1 flex items-center gap-2 \
                                           hover:bg-emerald-500/10 transition-colors border-t border-white/5"
                                >
                                    <span class="text-emerald-400 text-[10px] leading-none">"+"</span>
                                    <span class="text-white text-xs truncate flex-1">
                                        {move || input.get().trim().to_string()}
                                    </span>
                                    <span class="text-[9px] text-emerald-400 ml-auto shrink-0">
                                        "new tag"
                                    </span>
                                </button>
                            </Show>

                            <Show when=move || already_added.get() && suggestions.get().is_empty()>
                                <div class="px-2 py-1 text-[10px] text-gray-500 italic">
                                    "already attached"
                                </div>
                            </Show>
                        </div>
                    </Show>
                </div>
            </div>
        }
        .into_any()
    };

    view! {
        {render}
    }
}
