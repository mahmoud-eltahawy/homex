use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};

use crate::app::sections::{create_section, delete_section, fetch_sections};

pub struct SettingsPage;

#[lazy_route]
impl LazyRoute for SettingsPage {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        let sections = Resource::new(|| (), |_| fetch_sections());

        let create = Action::new_local(|(title, kind, nested): &(String, String, bool)| {
            create_section(title.clone(), kind.clone(), *nested)
        });
        let delete_action = Action::new_local(|id: &u64| delete_section(*id));

        Effect::new(move |_| {
            if matches!(create.value().get(), Some(Ok(_))) {
                sections.refetch();
            }
        });
        Effect::new(move |_| {
            if matches!(delete_action.value().get(), Some(Ok(_))) {
                sections.refetch();
            }
        });

        let title = RwSignal::new(String::new());
        let kind = RwSignal::new("video".to_string());
        let nested = RwSignal::new(false);

        let submit = move |ev: web_sys::SubmitEvent| {
            ev.prevent_default();
            let t = title.get_untracked();
            let k = kind.get_untracked();
            let n = nested.get_untracked();
            if t.is_empty() {
                return;
            }
            create.dispatch((t, k, n));
            title.set(String::new());
        };

        view! {
            <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-12 text-white">
                <h1 class="text-3xl font-black mb-8">"الإعدادات"</h1>
                <h2 class="text-xl font-bold mb-3">"الأقسام"</h2>

                <form on:submit=submit
                    class="bg-white/5 border border-white/10 rounded-2xl p-4 mb-8 flex flex-wrap gap-3 items-end">
                    <label class="flex flex-col text-sm flex-1 min-w-48">
                        <span class="mb-1">"الاسم"</span>
                        <input
                            type="text"
                            prop:value=move || title.get()
                            on:input=move |e| title.set(event_target_value(&e))
                            placeholder="أفلام، مسلسلات، ألبومات..."
                            class="bg-white/10 rounded-lg px-3 py-1.5 text-white w-full"
                        />
                    </label>
                    <label class="flex flex-col text-sm">
                        <span class="mb-1">"النوع"</span>
                        <select
                            prop:value=move || kind.get()
                            on:change=move |e| kind.set(event_target_value(&e))
                            class="bg-white/10 rounded-lg px-3 py-1.5 text-white">
                            <option value="video">"فيديو"</option>
                            <option value="audio">"صوت"</option>
                        </select>
                    </label>
                    <label class="flex items-center gap-2 text-sm pb-2">
                        <input
                            type="checkbox"
                            prop:checked=move || nested.get()
                            on:change=move |e| nested.set(event_target_checked(&e))
                        />
                        <span>"مجموعات"</span>
                    </label>
                    <button
                        type="submit"
                        class="px-4 py-2 rounded-xl bg-gradient-to-r from-cyan-500 to-blue-500 text-white font-bold text-sm">
                        "إضافة قسم"
                    </button>
                </form>

                <Transition fallback=|| view! { <div class="text-gray-500">"Loading..."</div> }>
                    {move || sections.get().map(|r| {
                        let list = r.unwrap_or_default();
                        if list.is_empty() {
                            return view! {
                                <div class="text-gray-500">"لا توجد أقسام بعد."</div>
                            }.into_any();
                        }
                        view! {
                            <ul class="space-y-2">
                                <For each=move || list.clone() key=|s| s.id let:section>
                                    <li class="flex items-center justify-between bg-white/5 rounded-xl px-4 py-3">
                                        <div class="min-w-0">
                                            <div class="font-bold truncate">{section.title.clone()}</div>
                                            <div class="text-xs text-gray-400">
                                                {format!(
                                                    "{} · {} · {}",
                                                    section.media_kind.label(),
                                                    if section.nested { "مجموعات" } else { "مفرد" },
                                                    section.slug,
                                                )}
                                            </div>
                                        </div>
                                        <button
                                            on:click={
                                                let id = section.id;
                                                move |_| { delete_action.dispatch(id); }
                                            }
                                            class="px-3 py-1.5 rounded-lg bg-red-500/20 hover:bg-red-500/30 text-red-300 text-sm shrink-0 ms-3">
                                            "حذف"
                                        </button>
                                    </li>
                                </For>
                            </ul>
                        }.into_any()
                    })}
                </Transition>
            </div>
        }
        .into_any()
    }
}
