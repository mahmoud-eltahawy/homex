use super::model::MediaType;
use crate::app::{
    icons::{
        AudioIcon, DeleteIcon, DownArrow, MovieIcon, SeriesIcon, SortIcon, UpArrow, UploadIcon,
    },
    resource_view::ResourceView,
};
use leptos::html;
use leptos::prelude::*;
use leptos_router::{lazy_route, LazyRoute};
use serde::{Deserialize, Serialize};
use server_fn::codec::{MultipartData, MultipartFormData};
use std::rc::Rc;
use std::time::Duration;
use web_sys::{
    wasm_bindgen::JsCast, FormData, HtmlFormElement, HtmlInputElement, HtmlSelectElement,
    MouseEvent,
};

const INPUT_CLASS: &str = "w-full bg-white/10 backdrop-blur-md text-white placeholder-gray-500 rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/20 transition";
const TEXTAREA_CLASS: &str = "w-full bg-white/10 backdrop-blur-md text-white placeholder-gray-500 rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/20 transition resize-none";
const CARD_CLASS: &str =
    "backdrop-blur-xl bg-white/5 rounded-3xl border border-white/10 p-6 md:p-8 shadow-2xl";
const ITEM_CARD_CLASS: &str = "bg-white/5 backdrop-blur-sm rounded-xl border border-white/10 p-4 flex flex-col sm:flex-row gap-3 items-start";
const TOOLBAR_BTN_CLASS: &str = "inline-flex items-center gap-1.5 bg-white/10 hover:bg-white/20 backdrop-blur-md text-white font-medium py-1.5 px-3 rounded-lg transition text-sm";
const UPLOAD_BTN_CLASS: &str = "inline-flex items-center gap-1.5 bg-green-500/20 hover:bg-green-500/30 backdrop-blur-md text-green-300 font-medium py-1.5 px-3 rounded-lg cursor-pointer transition text-sm";
const ICON_BTN_CLASS: &str = "text-gray-400 hover:text-white transition disabled:opacity-30 p-1";

// ─── Data types ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MediaTitle {
    pub id: u64,
    pub title: String,
}

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
}

// ─── Server functions (unchanged) ─────────────────────────────────────────

#[server]
async fn fetch_series_titles() -> Result<Vec<MediaTitle>, ServerFnError> {
    use crate::app::model::Series;
    use crate::app::{delay, mockary::mock_series};
    use std::collections::BTreeMap;
    delay(200).await;
    let mut map = BTreeMap::new();
    for Series { id, title, .. } in mock_series() {
        map.entry(id).or_insert(title);
    }
    Ok(map
        .into_iter()
        .map(|(id, title)| MediaTitle { id, title })
        .collect())
}

#[server]
async fn fetch_movie_titles() -> Result<Vec<MediaTitle>, ServerFnError> {
    use crate::app::model::Movie;
    use crate::app::{delay, mockary::mock_movies};
    use std::collections::BTreeMap;
    delay(200).await;
    let mut map = BTreeMap::new();
    for Movie { id, title, .. } in mock_movies() {
        map.entry(id).or_insert(title);
    }
    Ok(map
        .into_iter()
        .map(|(id, title)| MediaTitle { id, title })
        .collect())
}

#[server]
async fn fetch_audio_group_titles() -> Result<Vec<MediaTitle>, ServerFnError> {
    use crate::app::model::AudioGroup;
    use crate::app::{delay, mockary::mock_audio_groups};
    use std::collections::BTreeMap;
    delay(200).await;
    let mut map = BTreeMap::new();
    for AudioGroup { id, title, .. } in mock_audio_groups() {
        map.entry(id).or_insert(title);
    }
    Ok(map
        .into_iter()
        .map(|(id, title)| MediaTitle { id, title })
        .collect())
}

#[server(input = MultipartFormData)]
pub async fn upload_media(data: MultipartData) -> Result<UploadResult, ServerFnError> {
    use std::collections::BTreeMap;

    let mut multipart = data.into_inner().unwrap();

    let mut title = String::new();
    let mut media_type = String::new();
    let mut description = String::new();
    let mut is_new = true;
    let mut existing_id: Option<i64> = None;
    let mut season_number: Option<u32> = None;

    let mut files: BTreeMap<usize, (String, usize)> = BTreeMap::new();
    let mut file_titles: BTreeMap<usize, String> = BTreeMap::new();

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().map(String::from).unwrap_or_default();

        match name.as_str() {
            "title" => {
                title = field.text().await?;
                continue;
            }
            "media_type" => {
                media_type = field.text().await?;
                continue;
            }
            "description" => {
                description = field.text().await?;
                continue;
            }
            "is_new" => {
                is_new = field.text().await? == "true";
                continue;
            }
            "existing_id" => {
                let text = field.text().await?;
                existing_id = text.parse().ok();
                continue;
            }
            "season_number" => {
                let text = field.text().await?;
                season_number = text.parse().ok();
                continue;
            }
            _ => {}
        }

        if let Some(idx_str) = name.strip_prefix("file_title_") {
            if let Ok(idx) = idx_str.parse::<usize>() {
                file_titles.insert(idx, field.text().await?);
            } else {
                let _ = field.bytes().await?;
            }
        } else if let Some(idx_str) = name.strip_prefix("file_") {
            if let Ok(idx) = idx_str.parse::<usize>() {
                let file_name = field.file_name().map(String::from).unwrap_or_default();
                let bytes = field.bytes().await?;
                files.insert(idx, (file_name, bytes.len()));
            } else {
                let _ = field.bytes().await?;
            }
        } else {
            let _ = field.bytes().await?;
        }
    }

    if files.is_empty() {
        return Err(ServerFnError::new("لم يتم استلام أي ملف"));
    }

    let action = if is_new {
        "create new"
    } else {
        "append to existing"
    };
    let target = existing_id
        .map(|id| format!("id={id}"))
        .unwrap_or_else(|| "(no target)".to_string());
    leptos::logging::log!(
        "[upload] {action} {media_type} {target}: title={title:?} desc_len={} season={season_number:?} files={}",
        description.len(),
        files.len()
    );
    for (idx, (name, size)) in &files {
        let t = file_titles
            .get(idx)
            .map(String::as_str)
            .unwrap_or("(no title)");
        leptos::logging::log!("[upload]   file[{idx}]: {name} ({size} bytes) title={t:?}");
    }

    let verb = if is_new {
        "إنشاء"
    } else {
        "إضافة إلى"
    };
    Ok(UploadResult {
        success: true,
        message: format!("{verb} {} ملف بنجاح", files.len()),
    })
}

// ─── Page shell ───────────────────────────────────────────────────────────

pub struct UploadPage;

#[lazy_route]
impl LazyRoute for UploadPage {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! {
            <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
                <UploadHeader/>
                <div class=CARD_CLASS>
                    <UploadContent/>
                </div>
            </div>
        }
        .into_any()
    }
}

// ─── Main orchestrator ────────────────────────────────────────────────────

#[component]
fn UploadContent() -> impl IntoView {
    let media_type = RwSignal::new(None::<MediaType>);
    let is_new = RwSignal::new(None::<bool>);
    let existing_id = RwSignal::new(None::<i64>);

    // Details signals — lifted out of the form so they survive toggling.
    let title = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let season_number = RwSignal::new(1u32);

    // Files queued for the current operation.
    let items = RwSignal::new(Vec::<UploadItem>::new());
    let next_id = RwSignal::new(1u32);

    // Titles for the currently-selected media kind. Refetches on change.
    let titles = Resource::new(
        move || media_type.get(),
        |mt| async move {
            match mt {
                Some(MediaType::Series) => fetch_series_titles().await,
                Some(MediaType::Movie) => fetch_movie_titles().await,
                Some(MediaType::AudioGroup) => fetch_audio_group_titles().await,
                None => Ok(Vec::new()),
            }
        },
    );

    // Auto-scroll anchors.
    let s2_ref = NodeRef::<html::Div>::new();
    let s3_ref = NodeRef::<html::Div>::new();

    Effect::new(move |prev: Option<bool>| {
        let curr = media_type.get().is_some();
        if !prev.unwrap_or(false) && curr {
            let r = s2_ref;
            let _ = set_timeout(
                move || {
                    if let Some(el) = r.get() {
                        el.scroll_into_view();
                    }
                },
                Duration::from_millis(50),
            );
        }
        curr
    });

    Effect::new(move |prev: Option<bool>| {
        let curr = is_new.get().is_some();
        if !prev.unwrap_or(false) && curr {
            let r = s3_ref;
            let _ = set_timeout(
                move || {
                    if let Some(el) = r.get() {
                        el.scroll_into_view();
                    }
                },
                Duration::from_millis(50),
            );
        }
        curr
    });

    // Client-side validation. Returns the first unmet requirement, or None.
    let validation_hint = Signal::derive(move || -> Option<String> {
        let mt = match media_type.get() {
            Some(m) => m,
            None => return Some("اختر نوع الوسائط".into()),
        };
        let inw = match is_new.get() {
            Some(n) => n,
            None => return Some("اختر إنشاء جديد أو إضافة إلى موجود".into()),
        };
        if !inw && existing_id.get().is_none() {
            return Some("اختر العنصر الموجود".into());
        }
        if inw && title.get().trim().is_empty() {
            return Some("أدخل عنواناً".into());
        }
        if items.get().is_empty() {
            return Some(
                match mt {
                    MediaType::Movie => "أضف فصلاً واحداً على الأقل",
                    MediaType::Series => "أضف حلقة واحدة على الأقل",
                    MediaType::AudioGroup => "أضف مقطعاً صوتياً واحداً على الأقل",
                }
                .into(),
            );
        }
        None
    });

    let upload_action = Action::new_local(|data: &FormData| upload_media(data.clone().into()));

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();

        if validation_hint.get_untracked().is_some() {
            return;
        }

        let snapshot = items.get_untracked();

        let form = match ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlFormElement>().ok())
        {
            Some(f) => f,
            None => return,
        };

        let form_data = match FormData::new_with_form(&form) {
            Ok(fd) => fd,
            Err(_) => return,
        };

        // Files live in signals, not in the DOM, so inject them manually.
        for (i, item) in snapshot.iter().enumerate() {
            let _ = form_data.append_with_blob_and_filename(
                &format!("file_{i}"),
                &item.file,
                &item.file.name(),
            );
            let _ = form_data.append_with_str(&format!("file_title_{i}"), &item.title);
        }

        upload_action.dispatch(form_data);
    };

    let result_view = move || match upload_action.value().get() {
        Some(Ok(r)) => Some(view! {
            <div class="bg-green-500/15 text-green-300 border border-green-500/30 rounded-xl p-3 text-sm">
                {r.message}
            </div>
        }),
        Some(Err(e)) => Some(view! {
            <div class="bg-red-500/15 text-red-300 border border-red-500/30 rounded-xl p-3 text-sm">
                {e.to_string()}
            </div>
        }),
        None => None,
    };

    view! {
        <form on:submit=on_submit class="space-y-4 md:space-y-5">
            // ── ① Media type ──────────────────────────────────────────
            <FormSection
                number=1
                title="ما نوع الوسائط؟"
                done=Signal::derive(move || media_type.get().is_some())
            >
                <MediaKindCards
                    media_type
                    is_new
                    existing_id
                    title
                    description
                    season_number
                    items
                    next_id
                />
            </FormSection>

            // ── ② New vs existing ─────────────────────────────────────
            <Show when=move || media_type.get().is_some()>
                <div node_ref=s2_ref class="scroll-mt-24 md:scroll-mt-28">
                    <FormSection
                        number=2
                        title="إنشاء جديد أم إضافة إلى موجود؟"
                        done=Signal::derive(move || is_new.get().is_some())
                    >
                        <NewOrExistingCards is_new existing_id/>
                    </FormSection>
                </div>
            </Show>

            // ── ③ Details ─────────────────────────────────────────────
            <Show when=move || is_new.get().is_some()>
                <div node_ref=s3_ref class="scroll-mt-24 md:scroll-mt-28">
                    <FormSection
                        number=3
                        title="تفاصيل المحتوى"
                        done=Signal::derive(move || {
                            let inw = is_new.get();
                            if inw == Some(true) {
                                !title.get().trim().is_empty()
                            } else if inw == Some(false) {
                                existing_id.get().is_some()
                            } else {
                                false
                            }
                        })
                    >
                        <DetailsSection
                            media_type
                            is_new
                            existing_id
                            title
                            description
                            season_number
                            titles
                        />
                    </FormSection>
                </div>
            </Show>

            // ── ④ Files ───────────────────────────────────────────────
            <Show when=move || is_new.get().is_some()>
                <FormSection
                    number=4
                    title="الملفات"
                    done=Signal::derive(move || !items.get().is_empty())
                >
                    {move || match media_type.get() {
                        Some(MediaType::Movie) => view! {
                            <MediaFilesSection
                                items
                                next_id
                                heading="فصول الفيلم"
                                hint="سيتم إضافة الفصول الجديدة بعد آخر فصل موجود. استخدم الأسهم لإعادة الترتيب."
                                input_id="multiMovieInput"
                                accept="video/*"
                                select_label="اختيار فصول الفيلم"
                                number_label="رقم الفصل"
                                title_label="عنوان الفصل"
                                file_label="الملف"
                                icon=MovieIcon()
                            />
                        }
                        .into_any(),
                        Some(MediaType::Series) => view! {
                            <MediaFilesSection
                                items
                                next_id
                                heading="الحلقات"
                                hint="سيتم ترقيم الحلقات تلقائياً حسب الترتيب. استخدم الأسهم لإعادة الترتيب."
                                input_id="multiEpisodeInput"
                                accept="video/*"
                                select_label="اختيار الحلقات"
                                number_label="رقم الحلقة"
                                title_label="عنوان الحلقة"
                                file_label="الملف"
                                icon=SeriesIcon()
                            />
                        }
                        .into_any(),
                        Some(MediaType::AudioGroup) => view! {
                            <MediaFilesSection
                                items
                                next_id
                                heading="المقاطع الصوتية"
                                hint="سيتم إضافة المقاطع الجديدة في نهاية المجموعة. استخدم الأسهم لإعادة الترتيب."
                                input_id="multiAudioInput"
                                accept="audio/*"
                                select_label="اختيار ملفات صوتية"
                                number_label="رقم المقطع"
                                title_label="عنوان المقطع الصوتي"
                                file_label="الملف"
                                icon=AudioIcon()
                            />
                        }
                        .into_any(),
                        None => ().into_any(),
                    }}
                </FormSection>
            </Show>

            <HiddenFormState media_type is_new existing_id/>

            {result_view}

            <UploadSubmitButton
                pending=upload_action.pending().into()
                valid=Signal::derive(move || validation_hint.get().is_none())
                hint=Signal::derive(move || validation_hint.get().unwrap_or_default())
                file_count=Signal::derive(move || items.get().len())
            />
        </form>
    }
}

// ─── Section wrapper ──────────────────────────────────────────────────────

#[component]
fn FormSection(
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

// ─── Section ① — media kind ───────────────────────────────────────────────

#[component]
#[allow(clippy::too_many_arguments)]
fn MediaKindCards(
    media_type: RwSignal<Option<MediaType>>,
    is_new: RwSignal<Option<bool>>,
    existing_id: RwSignal<Option<i64>>,
    title: RwSignal<String>,
    description: RwSignal<String>,
    season_number: RwSignal<u32>,
    items: RwSignal<Vec<UploadItem>>,
    next_id: RwSignal<u32>,
) -> impl IntoView {
    // Single shared callback so all three cards get identical reset logic.
    let on_select: Rc<dyn Fn(MediaType)> = Rc::new(move |mt: MediaType| {
        if media_type.get_untracked() == Some(mt) {
            return;
        }
        // Guard against destroying queued work by accident.
        if !items.get_untracked().is_empty() {
            let confirmed = web_sys::window()
                .and_then(|w| {
                    w.confirm_with_message("سيتم حذف الملفات المضافة. هل تريد المتابعة؟")
                        .ok()
                })
                .unwrap_or(true);
            if !confirmed {
                return;
            }
        }
        batch(|| {
            media_type.set(Some(mt));
            is_new.set(None);
            existing_id.set(None);
            title.set(String::new());
            description.set(String::new());
            season_number.set(1);
            items.set(Vec::new());
            next_id.set(1);
        });
    });

    view! {
        <div class="grid grid-cols-3 gap-3">
            <MediaKindCard
                value=MediaType::Movie
                label="فيلم"
                icon=MovieIcon()
                media_type
                on_select=Rc::clone(&on_select)
            />
            <MediaKindCard
                value=MediaType::Series
                label="مسلسل"
                icon=SeriesIcon()
                media_type
                on_select=Rc::clone(&on_select)
            />
            <MediaKindCard
                value=MediaType::AudioGroup
                label="صوتيات"
                icon=AudioIcon()
                media_type
                on_select=on_select
            />
        </div>
    }
}

#[component]
fn MediaKindCard(
    value: MediaType,
    label: &'static str,
    icon: impl IntoView + 'static,
    media_type: RwSignal<Option<MediaType>>,
    on_select: Rc<dyn Fn(MediaType)>,
) -> impl IntoView {
    let is_active = move || media_type.get() == Some(value);
    let class = move || {
        format!(
            "group flex flex-col items-center justify-center gap-2 p-4 sm:p-6 rounded-2xl border-2 transition-all {}",
            if is_active() {
                "border-cyan-400 bg-cyan-500/10 shadow-lg shadow-cyan-500/20"
            } else {
                "border-white/10 bg-white/5 hover:border-white/20 hover:bg-white/10"
            }
        )
    };
    let on_click = {
        let on_select = Rc::clone(&on_select);
        move |_| on_select(value)
    };
    view! {
        <button type="button" on:click=on_click class=class>
            <div class="flex items-center justify-center w-10 h-10">{icon}</div>
            <div class="text-sm font-bold text-white">{label}</div>
        </button>
    }
}

// ─── Section ② — new vs existing ──────────────────────────────────────────

#[component]
fn NewOrExistingCards(
    is_new: RwSignal<Option<bool>>,
    existing_id: RwSignal<Option<i64>>,
) -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <ChoiceCard
                active=Signal::derive(move || is_new.get() == Some(true))
                label="إنشاء جديد"
                sub="ابدأ من الصفر"
                on_click=move |_: MouseEvent| {
                    is_new.set(Some(true));
                    existing_id.set(None);
                }
            />
            <ChoiceCard
                active=Signal::derive(move || is_new.get() == Some(false))
                label="إضافة إلى موجود"
                sub="أضف إلى عنصر موجود في المكتبة"
                on_click=move |_: MouseEvent| is_new.set(Some(false))
            />
        </div>
    }
}

#[component]
fn ChoiceCard<F>(
    #[prop(into)] active: Signal<bool>,
    label: &'static str,
    #[prop(optional)] sub: Option<&'static str>,
    on_click: F,
) -> impl IntoView
where
    F: Fn(MouseEvent) + 'static,
{
    let class = move || {
        format!(
            "flex flex-col items-start gap-1 p-4 rounded-2xl border-2 transition-all text-right {}",
            if active.get() {
                "border-cyan-400 bg-cyan-500/10 shadow-lg shadow-cyan-500/20"
            } else {
                "border-white/10 bg-white/5 hover:border-white/20 hover:bg-white/10"
            }
        )
    };
    view! {
        <button type="button" on:click=on_click class=class>
            <div class="text-sm font-bold text-white">{label}</div>
            {sub.map(|s| view! { <div class="text-xs text-gray-400">{s}</div> })}
        </button>
    }
}

// ─── Section ③ — details ──────────────────────────────────────────────────

#[component]
fn DetailsSection(
    media_type: RwSignal<Option<MediaType>>,
    is_new: RwSignal<Option<bool>>,
    existing_id: RwSignal<Option<i64>>,
    title: RwSignal<String>,
    description: RwSignal<String>,
    season_number: RwSignal<u32>,
    titles: Resource<Result<Vec<MediaTitle>, ServerFnError>>,
) -> impl IntoView {
    let is_new_true = Memo::new(move |_| is_new.get() == Some(true));
    let is_new_false = Memo::new(move |_| is_new.get() == Some(false));
    let is_series = Memo::new(move |_| media_type.get() == Some(MediaType::Series));

    view! {
        <div class="space-y-4">
            <Show when=move || is_new_true.get()>
                <TitleField title/>
                <DescriptionField description/>
            </Show>

            <Show when=move || is_new_false.get()>
                <ExistingSelectField
                    media_type=media_type.get()
                    existing_id
                    titles
                />
            </Show>

            <Show when=move || is_series.get()>
                <SeasonNumberField season_number/>
            </Show>
        </div>
    }
}

#[component]
fn TitleField(title: RwSignal<String>) -> impl IntoView {
    view! {
        <div>
            <label class="block text-sm font-medium text-gray-300 mb-1.5">"العنوان *"</label>
            <input
                type="text"
                name="title"
                prop:value=title
                on:input=move |ev| title.set(event_target_value(&ev))
                placeholder="أدخل العنوان..."
                class=INPUT_CLASS
            />
        </div>
    }
}

#[component]
fn DescriptionField(description: RwSignal<String>) -> impl IntoView {
    view! {
        <div>
            <label class="block text-sm font-medium text-gray-300 mb-1.5">
                "الوصف (اختياري)"
            </label>
            <textarea
                name="description"
                rows=3
                prop:value=description
                on:input=move |ev| description.set(event_target_value(&ev))
                placeholder="وصف مختصر (اختياري)..."
                class=TEXTAREA_CLASS
            ></textarea>
        </div>
    }
}

#[component]
fn SeasonNumberField(season_number: RwSignal<u32>) -> impl IntoView {
    view! {
        <div>
            <label class="block text-sm font-medium text-gray-300 mb-1.5">"رقم الموسم *"</label>
            <input
                type="number"
                name="season_number"
                min="1"
                prop:value=move || season_number.get().to_string()
                on:input=move |ev| {
                    if let Ok(n) = event_target_value(&ev).parse::<u32>() {
                        if n >= 1 {
                            season_number.set(n);
                        }
                    }
                }
                class=INPUT_CLASS
            />
        </div>
    }
}

#[component]
fn ExistingSelectField(
    media_type: Option<MediaType>,
    existing_id: RwSignal<Option<i64>>,
    titles: Resource<Result<Vec<MediaTitle>, ServerFnError>>,
) -> impl IntoView {
    let label = match media_type {
        Some(MediaType::Series) => "اختر المسلسل الموجود",
        Some(MediaType::Movie) => "اختر الفيلم الموجود",
        Some(MediaType::AudioGroup) => "اختر المجموعة الصوتية الموجودة",
        None => "اختر العنصر",
    };
    let adapter = move |list: Vec<MediaTitle>| ExistingSelectInnerProps { existing_id, list };
    view! {
        <div>
            <label class="block text-sm font-medium text-gray-300 mb-1.5">{label}</label>
            <ResourceView
                resource=titles
                view_fn=ExistingSelectInner
                adapter=adapter
            />
        </div>
    }
}

#[component]
fn ExistingSelectInner(existing_id: RwSignal<Option<i64>>, list: Vec<MediaTitle>) -> impl IntoView {
    view! {
        <select
            on:change=move |ev| {
                if let Some(sel) = ev
                    .target()
                    .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
                {
                    existing_id.set(sel.value().parse().ok());
                }
            }
            class=INPUT_CLASS
        >
            <option value="" class="bg-gray-800">
                "-- اختر --"
            </option>
            {list
                .into_iter()
                .map(|item| view! {
                    <option value={item.id.to_string()} class="bg-gray-800">
                        {item.title}
                    </option>
                })
                .collect_view()}
        </select>
    }
}

// ─── Section ④ — files ────────────────────────────────────────────────────

#[component]
fn MediaFilesSection(
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
            <MediaFilesToolbar
                items
                next_id
                heading
                input_id
                accept
                select_label
                icon
            />
            <MediaItemList
                items
                number_label
                title_label
                file_label
            />
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
                <MediaFilesInput
                    items
                    next_id
                    input_id
                    accept
                    select_label
                />
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
        {
            if let Some(files) = input.files() {
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
        }
    };

    view! {
        <input
            type="file"
            id=input_id
            class="hidden"
            multiple
            accept=accept
            on:change=file_handler
        />
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
                <MediaItemRow
                    items
                    item_id=item.id
                    number_label
                    title_label
                    file_label
                />
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
            if let Some(pos) = list.iter().position(|e| e.id == item_id) {
                if pos > 0 {
                    list.swap(pos, pos - 1);
                }
            }
        })
    };
    let move_down = move |_| {
        items.update(|list| {
            if let Some(pos) = list.iter().position(|e| e.id == item_id) {
                if pos + 1 < list.len() {
                    list.swap(pos, pos + 1);
                }
            }
        })
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
                    <input
                        type="text"
                        prop:value=title
                        on:input=title_update
                        placeholder=title_label
                        class="w-full bg-white/10 text-white rounded-lg py-1.5 px-3 text-sm focus:outline-none focus:ring-1 focus:ring-cyan-400"
                    />
                </div>
                <div class="hidden sm:block">
                    <span class="text-xs text-gray-400">{file_label}</span>
                    <div class="text-xs text-gray-300 truncate mt-0.5 max-w-32">
                        {file_name}
                    </div>
                </div>
            </div>
            <div class="flex items-center gap-1 mt-1 sm:mt-0">
                <button
                    type="button"
                    on:click=move_up
                    disabled=move || index() == 0
                    class=ICON_BTN_CLASS
                    title="نقل للأعلى"
                >
                    <UpArrow/>
                </button>
                <button
                    type="button"
                    on:click=move_down
                    disabled=move || index() + 1 == total()
                    class=ICON_BTN_CLASS
                    title="نقل للأسفل"
                >
                    <DownArrow/>
                </button>
                <button
                    type="button"
                    on:click=remove
                    class="text-red-400 hover:text-red-300 transition p-1"
                    title="حذف"
                >
                    <DeleteIcon/>
                </button>
            </div>
        </div>
    }
}

// ─── Hidden form state ────────────────────────────────────────────────────

#[component]
fn HiddenFormState(
    media_type: RwSignal<Option<MediaType>>,
    is_new: RwSignal<Option<bool>>,
    existing_id: RwSignal<Option<i64>>,
) -> impl IntoView {
    view! {
        <input
            type="hidden"
            name="media_type"
            value=move || media_type.get().map(|m| m.to_string()).unwrap_or_default()
        />
        <input
            type="hidden"
            name="is_new"
            value=move || is_new.get().map(|b| b.to_string()).unwrap_or_default()
        />
        <input
            type="hidden"
            name="existing_id"
            value=move || existing_id.get().map(|id| id.to_string()).unwrap_or_default()
        />
    }
}

// ─── Header & submit ──────────────────────────────────────────────────────

#[component]
fn UploadHeader() -> impl IntoView {
    view! {
        <div class="mb-8 md:mb-10 text-center">
            <div class="inline-flex items-center justify-center p-4 bg-cyan-400/10 rounded-3xl mb-4">
                <span class="text-cyan-400"><UploadIcon/></span>
            </div>
            <h1 class="text-3xl sm:text-4xl md:text-5xl font-black text-white">
                "رفع وسائط جديدة"
            </h1>
            <p class="text-gray-400 text-sm sm:text-base mt-2">
                "أضف فيلماً أو مسلسلاً أو مجموعة صوتية إلى مكتبتك المنزلية"
            </p>
        </div>
    }
}

#[component]
fn UploadSubmitButton(
    pending: Signal<bool>,
    valid: Signal<bool>,
    hint: Signal<String>,
    file_count: Signal<usize>,
) -> impl IntoView {
    let disabled = move || pending.get() || !valid.get();
    view! {
        <div class="space-y-2">
            <button
                type="submit"
                disabled=disabled
                class="w-full py-3 px-6 rounded-2xl bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold text-base shadow-lg shadow-cyan-500/20 transition-all hover:scale-[1.02] hover:shadow-cyan-500/40 flex items-center justify-center gap-2 disabled:opacity-40 disabled:cursor-not-allowed disabled:hover:scale-100 disabled:hover:from-cyan-500 disabled:hover:to-blue-500"
            >
                <UploadIcon/>
                {move || {
                    if pending.get() {
                        "جاري الرفع...".to_string()
                    } else {
                        let n = file_count.get();
                        if n == 0 {
                            "رفع الوسائط".to_string()
                        } else {
                            format!("رفع الوسائط ({n})")
                        }
                    }
                }}
            </button>
            <p class="text-center text-xs text-gray-500 h-4 leading-4">
                {move || hint.get()}
            </p>
        </div>
    }
}
