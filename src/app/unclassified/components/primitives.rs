use leptos::prelude::*;

use crate::app::unclassified::types::SectionWithCollections;

#[component]
pub fn ModalShell(
    #[prop(into)] title: String,
    subtitle: Signal<String>,
    busy: Signal<bool>,
    on_close: Callback<()>,
    children: Children,
) -> impl IntoView {
    let try_close = Callback::new(move |_: ()| {
        if !busy.get_untracked() {
            on_close.run(());
        }
    });

    view! {
        <div class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm \
                        flex items-center justify-center p-4"
            on:click=move |_| try_close.run(())>
            <div class="bg-[#12121a] border border-white/10 rounded-2xl p-6 w-full max-w-lg \
                            shadow-2xl max-h-[90vh] overflow-auto"
                on:click=|ev| ev.stop_propagation()>
                <div class="flex items-start justify-between gap-3 mb-4">
                    <div class="min-w-0">
                        <h3 class="text-white font-bold text-lg">{title}</h3>
                        <p class="text-gray-400 text-sm truncate mt-0.5">
                            {move || subtitle.get()}
                        </p>
                    </div>
                    <button type="button"
                        on:click=move |_| try_close.run(())
                        disabled=move || busy.get()
                        class="text-gray-400 hover:text-white transition shrink-0 \
                                   disabled:opacity-30 disabled:cursor-not-allowed"
                        aria-label="Close">"×"</button>
                </div>
                {children()}
            </div>
        </div>
    }
}

#[component]
pub fn TitleField(
    #[prop(default = "Title".to_string(), into)] label: String,
    value: RwSignal<String>,
    #[prop(into)] placeholder: String,
) -> impl IntoView {
    view! {
        <div>
            <label class="text-xs text-gray-500 uppercase tracking-wider">{label}</label>
            <input type="text"
                prop:value=move || value.get()
                on:input=move |e| value.set(event_target_value(&e))
                placeholder=placeholder
                class="mt-1 w-full bg-white/10 text-white rounded-lg px-3 py-1.5 text-sm \
                           focus:outline-none focus:ring-1 focus:ring-cyan-400"/>
        </div>
    }
}

#[component]
pub fn SectionSelect(
    options: Signal<Vec<SectionWithCollections>>,
    value: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div>
            <label class="text-xs text-gray-500 uppercase tracking-wider">"Section"</label>
            <select
                prop:value=move || value.get()
                on:change=move |e| value.set(event_target_value(&e))
                class="mt-1 w-full bg-white/10 text-white rounded-lg px-3 py-1.5 text-sm \
                           focus:outline-none focus:ring-1 focus:ring-cyan-400">
                <option value="">"Choose a section…"</option>
                {move || options.get().into_iter().map(|s| view! {
                    <option value=s.slug.to_string()>{s.title.to_string()}</option>
                }).collect_view()}
            </select>
        </div>
    }
}

#[component]
pub fn SeasonField(value: RwSignal<String>) -> impl IntoView {
    view! {
        <div>
            <label class="text-xs text-gray-500 uppercase tracking-wider">"Season"</label>
            <input type="number" min="1" placeholder="1"
                prop:value=move || value.get()
                on:input=move |e| value.set(event_target_value(&e))
                class="mt-1 w-full bg-white/10 text-white rounded-lg px-3 py-1.5 text-sm \
                           focus:outline-none focus:ring-1 focus:ring-cyan-400"/>
        </div>
    }
}

#[component]
pub fn FormActions(
    on_cancel: Callback<()>,
    on_confirm: Callback<()>,
    confirm_label: Signal<String>,
    disabled: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="flex justify-end gap-2 pt-2">
            <button type="button"
                on:click=move |_| on_cancel.run(())
                class="px-4 py-2 rounded-lg text-gray-300 hover:bg-white/10 transition text-sm">
                "Cancel"
            </button>
            <button type="button"
                on:click=move |_| on_confirm.run(())
                disabled=move || disabled.get()
                class="px-4 py-2 rounded-lg bg-gradient-to-r from-cyan-500 to-blue-500 \
                           hover:from-cyan-400 hover:to-blue-400 text-white font-bold text-sm \
                           disabled:opacity-50 transition">
                {move || confirm_label.get()}
            </button>
        </div>
    }
}
