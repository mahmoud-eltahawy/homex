use leptos::html;
use leptos::prelude::*;
use web_sys::HtmlInputElement;
use web_sys::wasm_bindgen::JsCast;

use crate::app::icons::{EditIcon, UploadIcon, XIcon};

pub const INPUT: &str = "w-full bg-white/10 backdrop-blur-md text-white rounded-lg py-1.5 px-3 focus:outline-none focus:ring-2 focus:ring-cyan-400/50";
pub const TEXTAREA: &str = "w-full bg-white/10 backdrop-blur-md text-white rounded-xl py-2 px-3 focus:outline-none focus:ring-2 focus:ring-cyan-400/50 resize-none";

const BTN_EDIT: &str = "p-1.5 rounded-lg bg-white/5 hover:bg-white/15 text-gray-300 hover:text-white transition shrink-0";
const BTN_OK: &str =
    "p-1.5 rounded-lg bg-cyan-500/20 hover:bg-cyan-500/30 text-cyan-300 transition shrink-0";
const BTN_X: &str =
    "p-1.5 rounded-lg bg-white/5 hover:bg-white/15 text-gray-300 transition shrink-0";

// ─── Single-line text ───────────────────────────────────────────────────────

#[component]
pub fn EditableText(
    value: Signal<String>,
    on_commit: Callback<String>,
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] placeholder: Option<String>,
) -> impl IntoView {
    let editing = RwSignal::new(false);
    let draft = RwSignal::new(String::new());
    let input_ref = NodeRef::<html::Input>::new();

    let display_class = class.unwrap_or_else(|| "text-white".into());
    let placeholder = placeholder.unwrap_or_else(|| "أدخل نصاً".into());

    let start = move |_| {
        draft.set(value.get_untracked());
        editing.set(true);
    };
    let save = move || {
        if !editing.get_untracked() {
            return;
        }
        editing.set(false);
        let new_val = draft.get_untracked();
        if new_val != value.get_untracked() {
            on_commit.run(new_val);
        }
    };
    let cancel = move || editing.set(false);

    Effect::new(move |_| {
        if editing.get()
            && let Some(input) = input_ref.get()
        {
            let _ = input.focus();
            input.select();
        }
    });

    let display_class_for_fallback = display_class.clone();
    let value_for_fallback = value;

    view! {
        <div class="inline-flex items-center gap-2 w-full min-w-0">
            <Show
                when=move || editing.get()
                fallback=move || {
                    let display_class = display_class_for_fallback.clone();
                    let value = value_for_fallback;
                    view! {
                        <>
                            <span class=display_class>{move || value.get()}</span>
                            <button type="button" class=BTN_EDIT on:click=start aria-label="تعديل">
                                <EditIcon/>
                            </button>
                        </>
                    }
                }
            >
                <input
                    node_ref=input_ref
                    type="text"
                    class=INPUT
                    placeholder=placeholder.clone()
                    prop:value=move || draft.get()
                    on:input=move |ev| draft.set(event_target_value(&ev))
                    on:keydown=move |ev| match ev.key().as_str() {
                        "Enter" => { ev.prevent_default(); save(); }
                        "Escape" => cancel(),
                        _ => {}
                    }
                />
                <button type="button" class=BTN_OK on:click=move |_| save() aria-label="حفظ">
                    "✓"
                </button>
                <button type="button" class=BTN_X on:click=move |_| cancel() aria-label="إلغاء">
                    <XIcon/>
                </button>
            </Show>
        </div>
    }
}

// ─── Multi-line text ────────────────────────────────────────────────────────

#[component]
pub fn EditableTextArea(
    value: Signal<String>,
    on_commit: Callback<String>,
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] placeholder: Option<String>,
) -> impl IntoView {
    let editing = RwSignal::new(false);
    let draft = RwSignal::new(String::new());
    let ta_ref = NodeRef::<html::Textarea>::new();

    let display_class = class.unwrap_or_else(|| "text-gray-300".into());
    let placeholder = placeholder.unwrap_or_else(|| "لا يوجد وصف".into());

    let start = move |_| {
        draft.set(value.get_untracked());
        editing.set(true);
    };
    let save = move || {
        if !editing.get_untracked() {
            return;
        }
        editing.set(false);
        let new_val = draft.get_untracked();
        if new_val != value.get_untracked() {
            on_commit.run(new_val);
        }
    };
    let cancel = move || editing.set(false);

    Effect::new(move |_| {
        if editing.get()
            && let Some(ta) = ta_ref.get()
        {
            let _ = ta.focus();
        }
    });

    // Clone once for the fallback so both the outer `Fn` fallback and the
    // inner reactive closure get their own copy.
    let display_class_for_fallback = display_class.clone();
    let placeholder_for_fallback = placeholder.clone();
    let value_for_fallback = value;

    view! {
        <div class="w-full">
            <Show
                when=move || editing.get()
                fallback=move || {
                    let display_class = display_class_for_fallback.clone();
                    let placeholder = placeholder_for_fallback.clone();
                    let value = value_for_fallback;
                    view! {
                        <div class="flex items-start gap-2">
                            <p class=display_class>
                                {move || {
                                    let v = value.get();
                                    if v.is_empty() { placeholder.clone() } else { v }
                                }}
                            </p>
                            <button type="button" class=BTN_EDIT on:click=start aria-label="تعديل">
                                <EditIcon/>
                            </button>
                        </div>
                    }
                }
            >
                <textarea
                    node_ref=ta_ref
                    rows=3
                    class=TEXTAREA
                    placeholder=placeholder.clone()
                    prop:value=move || draft.get()
                    on:input=move |ev| draft.set(event_target_value(&ev))
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" && (ev.ctrl_key() || ev.meta_key()) {
                            ev.prevent_default();
                            save();
                        } else if ev.key() == "Escape" {
                            cancel();
                        }
                    }
                ></textarea>
                <div class="flex gap-2 mt-2">
                    <button type="button" class=BTN_OK on:click=move |_| save() aria-label="حفظ">
                        "✓ حفظ"
                    </button>
                    <button type="button" class=BTN_X on:click=move |_| cancel() aria-label="إلغاء">
                        "إلغاء"
                    </button>
                </div>
            </Show>
        </div>
    }
}

// ─── Poster (upload/replace inline) ─────────────────────────────────────────

#[component]
pub fn EditablePoster(
    src: Signal<Option<String>>,
    placeholder: ViewFn,
    on_file: Callback<web_sys::File>,
    #[prop(into)] input_id: String,
) -> impl IntoView {
    let input_id_for_input = input_id.clone();
    let input_id_for_label = input_id;

    let on_change = move |ev: web_sys::Event| {
        let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        else {
            return;
        };
        let Some(files) = input.files() else { return };
        let Some(file) = files.get(0) else { return };
        input.set_value("");
        on_file.run(file);
    };

    view! {
        <div class="relative group w-full">
            {move || match src.get() {
                Some(url) => view! {
                    <img
                        src=url
                        class="w-full rounded-2xl shadow-2xl border border-white/10 object-cover"
                        alt=""
                    />
                }.into_any(),
                None => placeholder.run(),
            }}
            <input
                type="file"
                id=input_id_for_input
                class="hidden"
                accept="image/jpeg,image/png,image/webp,.jpg,.jpeg,.png,.webp"
                on:change=on_change
            />
            <label
                for=input_id_for_label
                class="absolute bottom-2 end-2 inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-black/60 backdrop-blur-md hover:bg-black/80 text-white text-xs font-medium cursor-pointer opacity-80 group-hover:opacity-100 transition"
            >
                <UploadIcon/> "تغيير الصورة"
            </label>
        </div>
    }
}
