use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};

use crate::app::{
    icons::{AudioIcon, EditIcon, MovieIcon, SeriesIcon, UploadIcon},
    model::MediaType,
    route_params::use_u64_param,
};

// ─── Page component ───────────────────────────────────────────────────────

#[component]
pub fn EditChooser(kind: MediaType, id: u64) -> impl IntoView {
    let back_href = kind.detail_href(id);
    let metadata_href = kind.edit_metadata_href(id);
    let append_href = kind.append_href(id);
    let kind_label = kind.label();
    let append_label = kind.append_label();
    let append_description = kind.append_description();

    let kind_icon = match kind {
        MediaType::Movie => MovieIcon().into_any(),
        MediaType::Series => SeriesIcon().into_any(),
        MediaType::AudioGroup => AudioIcon().into_any(),
    };

    view! {
        <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8 py-12 md:py-16">
            <a
                href=back_href
                class="inline-flex items-center gap-1 text-sm text-gray-400 hover:text-white transition mb-6"
            >
                <span class="text-cyan-400">"←"</span>
                <span>"رجوع"</span>
            </a>

            <div class="text-center mb-10 md:mb-12">
                <div class="inline-flex items-center justify-center p-4 bg-cyan-400/10 rounded-3xl mb-4">
                    <span class="text-cyan-400">{kind_icon}</span>
                </div>
                <h1 class="text-2xl sm:text-3xl md:text-4xl font-black text-white">
                    "ماذا تريد أن تفعل؟"
                </h1>
                <p class="text-gray-400 text-sm sm:text-base mt-2">
                    "تعديل بيانات "{kind_label}" أو إضافة محتوى جديد إليه"
                </p>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 md:gap-6">
                <ChooserCard
                    href=metadata_href
                    icon=EditIcon().into_any()
                    title="تعديل البيانات".to_string()
                    description="عدّل العنوان والوصف والصورة".to_string()
                />
                <ChooserCard
                    href=append_href
                    icon=UploadIcon().into_any()
                    title=append_label.to_string()
                    description=append_description.to_string()
                />
            </div>
        </div>
    }
}

#[component]
fn ChooserCard(
    #[prop(into)] href: String,
    icon: AnyView,
    #[prop(into)] title: String,
    #[prop(into)] description: String,
) -> impl IntoView {
    view! {
        <a
            href=href
            class="group flex flex-col items-center justify-center gap-3 p-8 md:p-10 rounded-3xl border-2 border-white/10 bg-white/5 hover:border-cyan-400 hover:bg-cyan-500/10 hover:shadow-lg hover:shadow-cyan-500/20 transition-all text-center min-h-[200px]"
        >
            <div class="flex items-center justify-center w-14 h-14 rounded-full bg-cyan-500/15 text-cyan-400 group-hover:bg-cyan-500/25 group-hover:scale-110 transition-all">
                {icon}
            </div>
            <div class="text-lg font-bold text-white">{title}</div>
            <div class="text-sm text-gray-400 leading-relaxed">{description}</div>
        </a>
    }
}

// ─── Route wrappers ───────────────────────────────────────────────────────

pub struct MovieEditChooser;
pub struct SeriesEditChooser;
pub struct AudioGroupEditChooser;

macro_rules! chooser_page {
    ($page:ident, $kind:expr) => {
        #[lazy_route]
        impl LazyRoute for $page {
            fn data() -> Self {
                Self
            }
            fn view(_this: Self) -> AnyView {
                view! {
                    <EditChooser kind=$kind id=use_u64_param("id")()/>
                }
                .into_any()
            }
        }
    };
}

chooser_page!(MovieEditChooser, MediaType::Movie);
chooser_page!(SeriesEditChooser, MediaType::Series);
chooser_page!(AudioGroupEditChooser, MediaType::AudioGroup);
