use leptos::prelude::*;
use leptos_router::{lazy_route, LazyRoute};

pub struct SettingsPage;

#[lazy_route]
impl LazyRoute for SettingsPage {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! {
            <div class="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-16">
                <div class="text-center text-white">
                    <h1 class="text-4xl font-black mb-4">الإعدادات</h1>
                    <p class="text-gray-400">سيتم إضافة صفحة الإعدادات قريباً.</p>
                </div>
            </div>
        }
        .into_any()
    }
}
