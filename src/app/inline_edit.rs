use leptos::either::Either;
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

#[derive(Clone, Copy)]
pub struct EditMode(pub RwSignal<bool>);

pub fn provide_edit_mode() -> RwSignal<bool> {
    let signal = RwSignal::new(false);
    provide_context(EditMode(signal));
    signal
}

pub fn use_edit_mode() -> Signal<bool> {
    match use_context::<EditMode>() {
        Some(EditMode(s)) => Signal::derive(move || s.get()),
        None => Signal::derive(|| true),
    }
}

pub fn collapse_when_locked(editing: RwSignal<bool>) {
    let mode_on = use_edit_mode();
    Effect::new(move |_| {
        if !mode_on.get() && editing.get() {
            editing.set(false);
        }
    });
}

#[derive(Clone, Copy)]
pub struct EditableCtl {
    pub commit: Callback<()>,
    pub cancel: Callback<()>,
}

#[component]
pub fn Editable<D, E>(
    #[prop(into)] display: Callback<(), D>,
    #[prop(into)] editor: Callback<EditableCtl, E>,
    #[prop(into)] on_commit: Callback<()>,
    #[prop(optional, into)] on_begin: Option<Callback<()>>,
    #[prop(default = true)] show_pencil: bool,
    #[prop(default = true)] show_actions: bool,
) -> impl IntoView
where
    D: IntoView + 'static,
    E: IntoView + 'static,
{
    let mode_on = use_edit_mode();
    let editing = RwSignal::new(false);
    collapse_when_locked(editing);

    let commit = Callback::new(move |_| {
        editing.set(false);
        on_commit.run(());
    });
    let cancel = Callback::new(move |_| editing.set(false));

    let begin = move |_| {
        if let Some(cb) = on_begin {
            cb.run(());
        }
        editing.set(true);
    };

    let ctl = EditableCtl { commit, cancel };

    view! {
        <div class="inline-flex items-center gap-2 w-full min-w-0">
            <Show
                when=move || editing.get()
                fallback=move || view! {
                    <>
                        {display.run(())}
                        {show_pencil.then(|| view! {
                            <Show when=move || mode_on.get()>
                                <button
                                    type="button"
                                    class=BTN_EDIT
                                    on:click=begin
                                    aria-label="تعديل"
                                >
                                    <EditIcon/>
                                </button>
                            </Show>
                        })}
                    </>
                }
            >
                {editor.run(ctl)}
                {show_actions.then(|| view! {
                    <button
                        type="button"
                        class=BTN_OK
                        on:click=move |_| ctl.commit.run(())
                        aria-label="حفظ"
                    >
                        "✓"
                    </button>
                    <button
                        type="button"
                        class=BTN_X
                        on:click=move |_| ctl.cancel.run(())
                        aria-label="إلغاء"
                    >
                        <XIcon/>
                    </button>
                })}
            </Show>
        </div>
    }
}

// ─── Single-line text ───────────────────────────────────────────────────────

#[component]
pub fn EditableText(
    value: Signal<String>,
    on_commit: Callback<String>,
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] placeholder: Option<String>,
) -> impl IntoView {
    let display_class = class.unwrap_or_else(|| "text-white".into());
    let placeholder = placeholder.unwrap_or_else(|| "أدخل نصاً".into());

    let draft = RwSignal::new(String::new());
    let input_ref = NodeRef::<html::Input>::new();

    // Seed the draft from the current value the moment editing starts.
    let on_begin = Callback::new(move |_| draft.set(value.get_untracked()));

    // Read the draft, fire the outer commit only if it actually changed.
    let commit = Callback::new(move |_| {
        let v = draft.get_untracked();
        if v != value.get_untracked() {
            on_commit.run(v);
        }
    });

    view! {
        <Editable
            on_begin=on_begin
            on_commit=commit
            display=Callback::new(move |_| view! {
                <span class=display_class.clone()>{move || value.get()}</span>
            })
            editor=Callback::new(move |ctl : EditableCtl| view! {
                <input
                    node_ref=input_ref
                    autofocus=true
                    type="text"
                    class=INPUT
                    placeholder=placeholder.clone()
                    prop:value=move || draft.get()
                    on:input=move |ev| draft.set(event_target_value(&ev))
                    on:keydown=move |ev| match ev.key().as_str() {
                        "Enter" => { ev.prevent_default(); ctl.commit.run(()); }
                        "Escape" => ctl.cancel.run(()),
                        _ => {}
                    }
                />
            })
        />
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
    let display_class = class.unwrap_or_else(|| "text-gray-300".into());
    let placeholder = placeholder.unwrap_or_else(|| "لا يوجد وصف".into());

    let draft = RwSignal::new(String::new());
    let ta_ref = NodeRef::<html::Textarea>::new();

    let on_begin = Callback::new(move |_| draft.set(value.get_untracked()));
    let commit = Callback::new(move |_| {
        let v = draft.get_untracked();
        if v != value.get_untracked() {
            on_commit.run(v);
        }
    });

    let display = Callback::new({
        let placeholder = placeholder.clone();
        move |_| {
            view! {
                <p class=display_class.clone()>
                    {
                        let placeholder = placeholder.clone();
                        move || {
                            let v = value.get();
                            if v.is_empty() { placeholder.clone() } else { v }
                        }
                    }
                </p>
            }
        }
    });

    let editor = Callback::new({
        let placeholder = placeholder.clone();
        move |ctl: EditableCtl| {
            view! {
                <textarea
                    node_ref=ta_ref
                    autofocus=true
                    rows=3
                    class=TEXTAREA
                    placeholder=placeholder.clone()
                    prop:value=move || draft.get()
                    on:input=move |ev| draft.set(event_target_value(&ev))
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" && (ev.ctrl_key() || ev.meta_key()) {
                            ev.prevent_default();
                            ctl.commit.run(());
                        } else if ev.key() == "Escape" {
                            ctl.cancel.run(());
                        }
                    }
                ></textarea>
            }
        }
    });
    view! {
        <Editable
            on_begin=on_begin
            on_commit=commit
            display=display
            editor=editor
        />
    }
}

// ─── Poster (upload/replace inline) ─────────────────────────────────────────

// ─── Poster (upload/replace inline) ─────────────────────────────────────────

#[component]
pub fn EditablePoster(
    src: Signal<Option<String>>,
    placeholder: ViewFn,
    on_file: Callback<web_sys::File>,
    #[prop(into)] input_id: String,
) -> impl IntoView {
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

    let edit_mode = use_edit_mode();

    let edit_view = {
        let input_id_for_input = input_id.clone();
        let input_id_for_label = input_id;
        move || {
            edit_mode.get().then_some(view! {
                <input
                    type="file"
                    id=input_id_for_input.clone()
                    class="hidden"
                    accept="image/jpeg,image/png,image/webp,.jpg,.jpeg,.png,.webp"
                    on:change=on_change
                />
                <label
                    for=input_id_for_label.clone()
                    class="absolute bottom-2 end-2 inline-flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-black/60 backdrop-blur-md hover:bg-black/80 text-white text-xs font-medium cursor-pointer opacity-80 group-hover:opacity-100 transition"
                >
                    <UploadIcon/> "تغيير الصورة"
                </label>
            })
        }
    };

    view! {
        <div class="relative group w-full">
            {move || match src.get() {
                Some(url) => Either::Left(view! {
                    <img
                        src=url
                        class="w-full rounded-2xl shadow-2xl border border-white/10 object-cover"
                        alt=""
                    />
                }),
                None => Either::Right(placeholder.run()),
            }}
            {edit_view}
        </div>
    }
}
