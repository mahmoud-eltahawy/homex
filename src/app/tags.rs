use leptos::prelude::*;

use crate::app::inline_edit::use_edit_mode;
use crate::app::model::Tag;

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

// ─── Tag editor ───────────────────────────────────────────────────────────
//
// `tags` is the source of truth for display. On every add/remove the signal
// is updated optimistically and `on_commit` is fired with the new set of
// names, so the server patches in the background and the UI stays snappy.
//
// Behaviour:
//   - view mode  → read-only pills (or "no tags" placeholder)
//   - edit mode  → pills with × button + an input at the end
//   - Enter      → commit the typed tag
//   - Backspace  → on an empty input, drop the last tag

#[component]
pub fn TagEditor(tags: RwSignal<Vec<Tag>>, on_commit: Callback<Vec<String>>) -> impl IntoView {
    let edit_mode = use_edit_mode();
    let input = RwSignal::new(String::new());

    let set_names = Callback::new(move |names: Vec<String>| {
        let new_tags: Vec<Tag> = names
            .iter()
            .map(|n| Tag {
                id: 0,
                name: n.clone(),
            })
            .collect();
        tags.set(new_tags);
        on_commit.run(names);
    });

    let add_current = Callback::new(move |_: ()| {
        let t = input.get_untracked().trim().to_string();
        if t.is_empty() {
            return;
        }
        let mut names: Vec<String> = tags.get_untracked().into_iter().map(|t| t.name).collect();
        input.set(String::new());
        if names.iter().any(|x| x.eq_ignore_ascii_case(&t)) {
            return;
        }
        names.push(t);
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
            add_current.run(());
        }
        "Backspace" if input.get_untracked().is_empty() => {
            let mut names: Vec<String> = tags.get_untracked().into_iter().map(|t| t.name).collect();
            if names.pop().is_some() {
                set_names.run(names);
            }
        }
        _ => {}
    });

    let render = move || {
        if edit_mode.get() {
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
                    <input
                        type="text"
                        placeholder="add tag..."
                        prop:value=move || input.get()
                        on:input=move |ev| input.set(event_target_value(&ev))
                        on:keydown=move |ev| on_keydown.run(ev)
                        class="bg-white/5 border border-white/10 rounded-full px-2 py-0.5 \
                               text-[10px] text-white placeholder:text-gray-500 \
                               focus:outline-none focus:border-cyan-400/40 w-24"
                    />
                </div>
            }
            .into_any()
        } else {
            let current = tags.get();
            if current.is_empty() {
                view! {
                    <span class="text-[10px] text-gray-500 italic">"no tags"</span>
                }
                .into_any()
            } else {
                view! {
                    <div class="flex flex-wrap items-center gap-1.5">
                        <For each=move || tags.get() key=|t| t.name.clone() let:tag>
                            <TagPill name=tag.name/>
                        </For>
                    </div>
                }
                .into_any()
            }
        }
    };

    view! {
        {render}
    }
}
