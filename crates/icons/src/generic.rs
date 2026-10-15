//! The shared `<svg>` wrapper every icon is built on.

use leptos::prelude::*;

/// The base icon wrapper. Renders a `24×24` viewBox `<svg>` with a `role`
/// and `aria-label` so icons are announced correctly to assistive tech.
///
/// `filled = true` switches between stroked and filled rendering; the two
/// are mutually exclusive because the original design used filled glyphs
/// with an inner knockout colour for the media icons.
#[component]
pub fn Icon(
    children: Children,
    #[prop(into)] class: String,
    #[prop(into)] label: String,
    #[prop(default = false)] filled: bool,
    #[prop(default = "1.5")] stroke_width: &'static str,
) -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            class=class
            viewBox="0 0 24 24"
            fill=if filled { "currentColor" } else { "none" }
            stroke=if !filled { "currentColor" } else { "none" }
            stroke-width=stroke_width
            stroke-linecap="round"
            stroke-linejoin="round"
            role="img"
            aria-label=label.clone()
        >
            {children()}
        </svg>
    }
}
