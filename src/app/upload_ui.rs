use leptos::either::Either;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use web_sys::{HtmlInputElement, Url, wasm_bindgen::JsCast};

use crate::app::icons::{DeleteIcon, DownArrow, SortIcon, UpArrow, UploadIcon};

// ─── Classes ──────────────────────────────────────────────────────────────

pub const INPUT_CLASS: &str = "w-full bg-white/10 backdrop-blur-md text-white placeholder-gray-500 rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/20 transition";
pub const TEXTAREA_CLASS: &str = "w-full bg-white/10 backdrop-blur-md text-white placeholder-gray-500 rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/20 transition resize-none";
pub const CARD_CLASS: &str =
    "backdrop-blur-xl bg-white/5 rounded-3xl border border-white/10 p-6 md:p-8 shadow-2xl";
const ITEM_CARD_CLASS: &str = "bg-white/5 backdrop-blur-sm rounded-xl border border-white/10 p-4 flex flex-col sm:flex-row gap-3 items-start";
const TOOLBAR_BTN_CLASS: &str = "inline-flex items-center gap-1.5 bg-white/10 hover:bg-white/20 backdrop-blur-md text-white font-medium py-1.5 px-3 rounded-lg transition text-sm";
const UPLOAD_BTN_CLASS: &str = "inline-flex items-center gap-1.5 bg-green-500/20 hover:bg-green-500/30 backdrop-blur-md text-green-300 font-medium py-1.5 px-3 rounded-lg cursor-pointer transition text-sm";
const ICON_BTN_CLASS: &str = "text-gray-400 hover:text-white transition disabled:opacity-30 p-1";

pub const VIDEO_ACCEPT: &str = "video/mp4,video/webm,video/x-matroska,video/quicktime,\
video/x-msvideo,video/x-ms-wmv,video/x-flv,video/mp2t,\
.mp4,.m4v,.webm,.mkv,.mov,.avi,.wmv,.flv,.ts";

pub const AUDIO_ACCEPT: &str = "audio/mpeg,audio/mp4,audio/aac,audio/wav,audio/x-wav,\
audio/ogg,audio/opus,audio/flac,audio/x-flac,audio/x-ms-wma,audio/aiff,\
.mp3,.m4a,.aac,.wav,.ogg,.oga,.opus,.flac,.wma,.aiff,.aif";

const IMAGE_ACCEPT: &str = "image/jpeg,image/png,image/webp,.jpg,.jpeg,.png,.webp";

// ─── DTOs ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct UploadItem {
    pub id: u32,
    pub file: web_sys::File,
    pub title: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UploadResult {
    pub success: bool,
    pub message: String,
    pub job_id: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ConversionStatus {
    Writing,
    Converting {
        conversion_index: usize,
        conversion_count: usize,
        current_file: String,
        progress: f32,
    },
    Finalizing,
    Done,
    Failed(String),
}

// ─── Field components ─────────────────────────────────────────────────────

#[component]
pub fn TitleField(title: RwSignal<String>) -> impl IntoView {
    view! {
        <div>
            <label class="block text-sm font-medium text-gray-300 mb-1.5">"العنوان *"</label>
            <input type="text" name="title"
                prop:value=title
                on:input=move |ev| title.set(event_target_value(&ev))
                placeholder="أدخل العنوان..." class=INPUT_CLASS/>
        </div>
    }
}

#[component]
pub fn DescriptionField(description: RwSignal<String>) -> impl IntoView {
    view! {
        <div>
            <label class="block text-sm font-medium text-gray-300 mb-1.5">
                "الوصف (اختياري)"
            </label>
            <textarea name="description" rows=3
                prop:value=description
                on:input=move |ev| description.set(event_target_value(&ev))
                placeholder="وصف مختصر (اختياري)..." class=TEXTAREA_CLASS
            ></textarea>
        </div>
    }
}

#[component]
pub fn SeasonNumberField(season_number: RwSignal<u32>) -> impl IntoView {
    view! {
        <div>
            <label class="block text-sm font-medium text-gray-300 mb-1.5">"رقم الموسم *"</label>
            <input type="number" name="season_number" min="1"
                prop:value=move || season_number.get().to_string()
                on:input=move |ev| {
                    if let Ok(n) = event_target_value(&ev).parse::<u32>() && n >= 1 {
                        season_number.set(n);
                    }
                }
                class=INPUT_CLASS/>
        </div>
    }
}

#[component]
pub fn PosterUploadField(poster_file: RwSignal<Option<web_sys::File>>) -> impl IntoView {
    let input_id = "posterFileInput";
    let preview_url = RwSignal::new(None::<String>);
    let file_name = RwSignal::new(String::new());

    let on_change = move |ev: web_sys::Event| {
        let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        else {
            return;
        };
        let Some(files) = input.files() else { return };
        let Some(file) = files.get(0) else { return };

        if let Some(prev) = preview_url.get_untracked() {
            let _ = Url::revoke_object_url(&prev);
        }

        file_name.set(file.name());
        if let Ok(url) = Url::create_object_url_with_blob(&file) {
            preview_url.set(Some(url));
        } else {
            preview_url.set(None);
        }
        poster_file.set(Some(file));
    };

    let clear = move |_| {
        if let Some(prev) = preview_url.get_untracked() {
            let _ = Url::revoke_object_url(&prev);
        }
        preview_url.set(None);
        file_name.set(String::new());
        poster_file.set(None);
    };

    on_cleanup(move || {
        if let Some(prev) = preview_url.get_untracked() {
            let _ = Url::revoke_object_url(&prev);
        }
    });

    view! {
        <div>
            <label class="block text-sm font-medium text-gray-300 mb-1.5">
                "الصورة (اختياري)"
            </label>
            <div class="flex items-start gap-4">
                <div class="w-24 h-36 rounded-xl border border-white/10 bg-white/5 flex items-center justify-center overflow-hidden shrink-0">
                    {move || match preview_url.get() {
                        Some(url) => Either::Left(view! {
                            <img src=url class="w-full h-full object-cover" alt=""/>
                        }),
                        None => Either::Right(view! {
                            <span class="text-gray-600 text-xs text-center px-2">
                                "لا توجد صورة"
                            </span>
                        }),
                    }}
                </div>
                <div class="flex-1 flex flex-col gap-2 min-w-0">
                    <input type="file" id=input_id class="hidden"
                        accept=IMAGE_ACCEPT on:change=on_change/>
                    <label for=input_id class=UPLOAD_BTN_CLASS>
                        <UploadIcon/> "اختيار صورة"
                    </label>
                    <Show when=move || !file_name.get().is_empty()>
                        <div class="flex items-center gap-2 text-xs text-gray-400">
                            <span class="truncate">{move || file_name.get()}</span>
                            <button type="button" on:click=clear
                                class="text-red-400 hover:text-red-300 transition shrink-0"
                                aria-label="إزالة الصورة">
                                <DeleteIcon/>
                            </button>
                        </div>
                    </Show>
                    <p class="text-xs text-gray-500">
                        "تظهر في القوائم وصفحة التفاصيل. صورة عمودية (2:3) للأفلام والمسلسلات، مربعة للصوتيات."
                    </p>
                </div>
            </div>
        </div>
    }
}

// ─── Section wrapper ──────────────────────────────────────────────────────

#[component]
pub fn FormSection(
    number: u32,
    title: &'static str,
    #[prop(into)] done: Signal<bool>,
    children: Children,
) -> impl IntoView {
    let badge_class = move || {
        format!(
            "flex items-center justify-center w-8 h-8 rounded-full text-sm font-bold transition-colors shrink-0 {}",
            if done.get() {
                "bg-green-500/20 text-green-400"
            } else {
                "bg-cyan-500/20 text-cyan-400"
            }
        )
    };
    view! {
        <section class="bg-white/[0.02] border border-white/5 rounded-2xl p-5 md:p-6">
            <div class="flex items-center gap-3 mb-5">
                <div class=badge_class>
                    {move || if done.get() { "✓".to_string() } else { number.to_string() }}
                </div>
                <h2 class="text-base sm:text-lg font-bold text-white">{title}</h2>
            </div>
            <div>{children()}</div>
        </section>
    }
}

// ─── File list editor ─────────────────────────────────────────────────────

#[component]
pub fn MediaFilesSection(
    items: RwSignal<Vec<UploadItem>>,
    next_id: RwSignal<u32>,
    heading: &'static str,
    hint: &'static str,
    input_id: &'static str,
    accept: &'static str,
    select_label: &'static str,
    number_label: &'static str,
    title_label: &'static str,
    file_label: &'static str,
    icon: impl IntoView,
) -> impl IntoView {
    view! {
        <div class="space-y-4">
            <MediaFilesToolbar items next_id heading input_id accept select_label icon/>
            <MediaItemList items number_label title_label file_label/>
            <p class="text-xs text-gray-500">{hint}</p>
        </div>
    }
}

#[component]
fn MediaFilesToolbar(
    items: RwSignal<Vec<UploadItem>>,
    next_id: RwSignal<u32>,
    heading: &'static str,
    input_id: &'static str,
    accept: &'static str,
    select_label: &'static str,
    icon: impl IntoView,
) -> impl IntoView {
    view! {
        <div class="flex flex-wrap items-center justify-between gap-3">
            <h2 class="text-lg font-bold text-white flex items-center gap-2">
                {icon} {heading}
            </h2>
            <div class="flex flex-wrap items-center gap-2">
                <MediaFilesInput items next_id input_id accept select_label/>
                <SortMediaButton items/>
            </div>
        </div>
    }
}

#[component]
fn MediaFilesInput(
    items: RwSignal<Vec<UploadItem>>,
    next_id: RwSignal<u32>,
    input_id: &'static str,
    accept: &'static str,
    select_label: &'static str,
) -> impl IntoView {
    let file_handler = move |ev: web_sys::Event| {
        if let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
            && let Some(files) = input.files()
        {
            let mut new_items: Vec<UploadItem> = (0..files.length())
                .filter_map(|i| files.get(i))
                .map(|file| {
                    let name = file.name();
                    let title = name.rsplitn(2, '.').last().unwrap_or(&name).to_string();
                    UploadItem {
                        id: next_id.get(),
                        file,
                        title,
                    }
                })
                .collect();

            new_items.sort_by_key(|x| x.file.name());
            let added = new_items.len() as u32;
            items.update(|list| list.extend(new_items));
            next_id.update(|id| *id += added);
            input.set_value("");
        }
    };

    view! {
        <input type="file" id=input_id class="hidden"
            multiple accept=accept on:change=file_handler/>
        <label for=input_id class=UPLOAD_BTN_CLASS>
            <UploadIcon/> {select_label}
        </label>
    }
}

#[component]
fn SortMediaButton(items: RwSignal<Vec<UploadItem>>) -> impl IntoView {
    let sort = move |_| items.update(|list| list.sort_by_key(|x| x.file.name()));
    view! {
        <button type="button" on:click=sort class=TOOLBAR_BTN_CLASS>
            <SortIcon/> "ترتيب"
        </button>
    }
}

#[component]
fn MediaItemList(
    items: RwSignal<Vec<UploadItem>>,
    number_label: &'static str,
    title_label: &'static str,
    file_label: &'static str,
) -> impl IntoView {
    view! {
        <div class="space-y-3 max-h-80 overflow-y-auto p-1">
            <For each=move || items.get() key=|item| item.id let:item>
                <MediaItemRow items item_id=item.id number_label title_label file_label/>
            </For>
        </div>
    }
}

#[component]
fn MediaItemRow(
    items: RwSignal<Vec<UploadItem>>,
    item_id: u32,
    number_label: &'static str,
    title_label: &'static str,
    file_label: &'static str,
) -> impl IntoView {
    let index = move || items.with(|list| list.iter().position(|e| e.id == item_id).unwrap_or(0));
    let total = move || items.with(|list| list.len());
    let title = move || {
        items.with(|list| {
            list.iter()
                .find(|e| e.id == item_id)
                .map(|e| e.title.clone())
                .unwrap_or_default()
        })
    };
    let file_name = move || {
        items.with(|list| {
            list.iter()
                .find(|e| e.id == item_id)
                .map(|e| e.file.name())
                .unwrap_or_default()
        })
    };

    let remove = move |_| items.update(|list| list.retain(|e| e.id != item_id));
    let move_up = move |_| {
        items.update(|list| {
            if let Some(pos) = list.iter().position(|e| e.id == item_id)
                && pos > 0
            {
                list.swap(pos, pos - 1);
            }
        });
    };
    let move_down = move |_| {
        items.update(|list| {
            if let Some(pos) = list.iter().position(|e| e.id == item_id)
                && pos + 1 < list.len()
            {
                list.swap(pos, pos + 1);
            }
        });
    };
    let title_update = move |ev: web_sys::Event| {
        if let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        {
            let val = input.value();
            items.update(|list| {
                if let Some(item) = list.iter_mut().find(|e| e.id == item_id) {
                    item.title = val;
                }
            });
        }
    };

    view! {
        <div class=ITEM_CARD_CLASS>
            <div class="flex-1 grid grid-cols-1 sm:grid-cols-3 gap-3 w-full">
                <div>
                    <span class="text-gray-400 text-sm font-medium">{number_label}</span>
                    <div class="text-white font-semibold mt-0.5">{move || index() + 1}</div>
                </div>
                <div class="sm:col-span-2">
                    <label class="text-xs text-gray-400 mb-0.5 block">{title_label}</label>
                    <input type="text" prop:value=title on:input=title_update
                        placeholder=title_label
                        class="w-full bg-white/10 text-white rounded-lg py-1.5 px-3 text-sm focus:outline-none focus:ring-1 focus:ring-cyan-400"/>
                </div>
                <div class="hidden sm:block">
                    <span class="text-xs text-gray-400">{file_label}</span>
                    <div class="text-xs text-gray-300 truncate mt-0.5 max-w-32">{file_name}</div>
                </div>
            </div>
            <div class="flex items-center gap-1 mt-1 sm:mt-0">
                <button type="button" on:click=move_up disabled=move || index() == 0
                    class=ICON_BTN_CLASS title="نقل للأعلى"><UpArrow/></button>
                <button type="button" on:click=move_down
                    disabled=move || index() + 1 == total()
                    class=ICON_BTN_CLASS title="نقل للأسفل"><DownArrow/></button>
                <button type="button" on:click=remove
                    class="text-red-400 hover:text-red-300 transition p-1" title="حذف">
                    <DeleteIcon/>
                </button>
            </div>
        </div>
    }
}

// ─── Header, progress, submit ────────────────────────────────────────────

#[component]
pub fn UploadHeader(
    #[prop(into)] title: String,
    #[prop(into)] subtitle: String,
    icon: impl IntoView + 'static,
) -> impl IntoView {
    view! {
        <div class="mb-8 md:mb-10 text-center">
            <div class="inline-flex items-center justify-center p-4 bg-cyan-400/10 rounded-3xl mb-4">
                <span class="text-cyan-400">{icon}</span>
            </div>
            <h1 class="text-3xl sm:text-4xl md:text-5xl font-black text-white">{title}</h1>
            <p class="text-gray-400 text-sm sm:text-base mt-2">{subtitle}</p>
        </div>
    }
}

#[component]
pub fn ConversionProgress(status: ConversionStatus) -> impl IntoView {
    match status {
    ConversionStatus::Writing => view! {
        <ProgressShell icon="💾" title="جاري حفظ الملفات على القرص..."
            subtitle=String::new() percent=None/>
    }.into_any(),

    ConversionStatus::Converting { conversion_index, conversion_count, current_file, progress } => {
        let pct = (progress * 100.0).round() as u32;
        let subtitle = if conversion_count > 1 {
            format!("ملف {} من {} — {}", conversion_index + 1, conversion_count, current_file)
        } else {
            current_file
        };
        view! {
            <ProgressShell icon="🎬" title="جاري تحويل الملف..."
                subtitle=subtitle percent=Some(pct)/>
        }.into_any()
    }

    ConversionStatus::Finalizing => view! {
        <ProgressShell icon="💾" title="جاري حفظ البيانات..."
            subtitle=String::new() percent=None/>
    }.into_any(),

    ConversionStatus::Done => view! {
        <div class="bg-green-500/15 border border-green-500/30 rounded-xl p-4 text-green-300 text-sm flex items-center gap-3">
            <span class="text-lg">"✓"</span>
            <span>"تم رفع الوسائط وتحويلها بنجاح"</span>
        </div>
    }.into_any(),

    ConversionStatus::Failed(err) => view! {
        <div class="bg-red-500/15 border border-red-500/30 rounded-xl p-4 text-red-300 text-sm">
            <div class="font-bold mb-1">"فشل التحويل"</div>
            <div class="text-xs break-all">{err}</div>
        </div>
    }.into_any(),
}
}

#[component]
fn ProgressShell(
    icon: &'static str,
    title: &'static str,
    #[prop(into)] subtitle: String,
    percent: Option<u32>,
) -> impl IntoView {
    let show_percent = percent.is_some();
    let percent = percent.unwrap_or(0);

    view! {
        <div class="bg-cyan-500/10 border border-cyan-500/30 rounded-xl p-4 space-y-3">
            <div class="flex items-center gap-3 text-cyan-300">
                <span class="text-lg">{icon}</span>
                <span class="font-bold text-sm">{title}</span>
            </div>
            <div class="text-xs text-gray-400 truncate font-mono min-h-4">{subtitle}</div>
            {show_percent.then(|| view! {
                <div class="space-y-1">
                    <div class="w-full h-2 bg-white/10 rounded-full overflow-hidden">
                        <div class="h-full bg-gradient-to-r from-cyan-400 to-blue-500 rounded-full transition-all duration-300"
                            style=format!("width: {percent}%")></div>
                    </div>
                    <div class="text-xs text-cyan-300 font-mono text-right">{percent}"%"</div>
                </div>
            })}
        </div>
    }
}

#[component]
pub fn UploadSubmitButton(
    pending: Signal<bool>,
    valid: Signal<bool>,
    hint: Signal<String>,
    file_count: Signal<usize>,
    #[prop(into)] label: String,
) -> impl IntoView {
    let disabled = move || pending.get() || !valid.get();
    view! {
        <div class="space-y-2">
            <button type="submit" disabled=disabled
                class="w-full py-3 px-6 rounded-2xl bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold text-base shadow-lg shadow-cyan-500/20 transition-all hover:scale-[1.02] hover:shadow-cyan-500/40 flex items-center justify-center gap-2 disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:scale-100 disabled:hover:from-cyan-500 disabled:hover:to-blue-500">
                <UploadIcon/>
                {move || {
                    if pending.get() {
                        "جاري الرفع...".to_string()
                    } else {
                        let n = file_count.get();
                        if n == 0 { label.clone() }
                        else { format!("{label} ({n})") }
                    }
                }}
            </button>
            <p class="text-center text-xs text-gray-500 h-4 leading-4">
                {move || hint.get()}
            </p>
        </div>
    }
}
