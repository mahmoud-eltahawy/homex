use leptos::prelude::*;

use crate::app::model::Tag;

// ─── Tag pill ─────────────────────────────────────────────────────────────
//
// Small rounded chip. Sized for card and inline usage; the detail page
// renders the same pill in a looser row and it still reads fine.

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

// ─── Tag list ─────────────────────────────────────────────────────────────
//
// `max = None` renders every tag. `max = Some(n)` renders the first `n` and
// a `+k` overflow chip. Renders nothing when the list is empty, so call
// sites can drop it in unconditionally.

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
