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

        let create = Action::new_local(
            |(slug, title, kind, nested): &(String, String, String, bool)| {
                create_section(slug.clone(), title.clone(), kind.clone(), *nested)
            },
        );
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

        let slug = RwSignal::new(String::new());
        let title = RwSignal::new(String::new());
        let kind = RwSignal::new("video".to_string());
        let nested = RwSignal::new(false);

        let submit = move |ev: web_sys::SubmitEvent| {
            ev.prevent_default();
            let s = slug.get_untracked();
            let t = title.get_untracked();
            let k = kind.get_untracked();
            let n = nested.get_untracked();
            if s.is_empty() || t.is_empty() {
                return;
            }
            create.dispatch((s, t, k, n));
            slug.set(String::new());
            title.set(String::new());
        };

        view! {
            <div class="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 py-12 text-white">
                <h1 class="text-3xl font-black mb-8">"الإعدادات"</h1>
                <h2 class="text-xl font-bold mb-3">"الأقسام"</h2>

                <form on:submit=submit
                    class="bg-white/5 border border-white/10 rounded-2xl p-4 mb-8 flex flex-wrap gap-3 items-end">
                    <label class="flex flex-col text-sm">
                        <span class="mb-1">"المعرّف (slug)"</span>
                        <input type="text" prop:value=move || slug.get()
                            on:input=move |e| slug.set(event_target_value(&e))
                            class="bg-white/10 rounded-lg px-3 py-1.5 text-white min-w-32"/>
                    </label>
                    <label class="flex flex-col text-sm">
                        <span class="mb-1">"العنوان"</span>
                        <input type="text" prop:value=move || title.get()
                            on:input=move |e| title.set(event_target_value(&e))
                            class="bg-white/10 rounded-lg px-3 py-1.5 text-white min-w-32"/>
                    </label>
                    <label class="flex flex-col text-sm">
                        <span class="mb-1">"النوع"</span>
                        <select prop:value=move || kind.get()
                            on:change=move |e| kind.set(event_target_value(&e))
                            class="bg-white/10 rounded-lg px-3 py-1.5 text-white">
                            <option value="video">"فيديو"</option>
                            <option value="audio">"صوت"</option>
                        </select>
                    </label>
                    <label class="flex items-center gap-2 text-sm pb-2">
                        <input type="checkbox" prop:checked=move || nested.get()
                            on:change=move |e| nested.set(event_target_checked(&e))/>
                        <span>"مجموعات"</span>
                    </label>
                    <button type="submit"
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
                                        <div>
                                            <div class="font-bold">{section.title.clone()}</div>
                                            <div class="text-xs text-gray-400">
                                                {format!(
                                                    "/s/{} — {} — {}",
                                                    section.slug,
                                                    section.media_kind.as_str(),
                                                    if section.nested { "مجموعات" } else { "مفرد" }
                                                )}
                                            </div>
                                        </div>
                                        <button
                                            on:click={
                                                let id = section.id;
                                                move |_| {delete_action.dispatch(id);}
                                            }
                                            class="px-3 py-1.5 rounded-lg bg-red-500/20 hover:bg-red-500/30 text-red-300 text-sm">
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
