use super::model::MediaType;
use crate::app::icons::UploadIcon;
use crate::app::{
    icons::{AudioIcon, MovieIcon, SeriesIcon},
    resource_view::ResourceView,
    upload_api::{poll_conversion, upload_media},
    upload_ui::{
        AUDIO_ACCEPT, CARD_CLASS, ConversionProgress, ConversionStatus, DescriptionField,
        FormSection, INPUT_CLASS, MediaFilesSection, PosterUploadField, SeasonNumberField,
        TitleField, UploadHeader, UploadItem, UploadSubmitButton, VIDEO_ACCEPT,
    },
};
use gloo_timers::future::TimeoutFuture;
use leptos::html;
use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use std::time::Duration;
use web_sys::{FormData, HtmlFormElement, HtmlSelectElement, MouseEvent, wasm_bindgen::JsCast};

// ─── DTOs ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MediaTitle {
    pub id: u64,
    pub title: String,
}

// ─── Server functions ─────────────────────────────────────────────────────

#[server]
async fn fetch_series_titles() -> Result<Vec<MediaTitle>, ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();

    #[derive(sqlx::FromRow)]
    struct Row {
        id: i64,
        title: String,
    }

    let rows: Vec<Row> = sqlx::query_as("SELECT id, title FROM series ORDER BY title")
        .fetch_all(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|r| MediaTitle {
            id: r.id as u64,
            title: r.title,
        })
        .collect())
}

#[server]
async fn fetch_movie_titles() -> Result<Vec<MediaTitle>, ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();

    #[derive(sqlx::FromRow)]
    struct Row {
        id: i64,
        title: String,
    }

    let rows: Vec<Row> = sqlx::query_as("SELECT id, title FROM movies ORDER BY title")
        .fetch_all(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|r| MediaTitle {
            id: r.id as u64,
            title: r.title,
        })
        .collect())
}

#[server]
async fn fetch_audio_group_titles() -> Result<Vec<MediaTitle>, ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();

    #[derive(sqlx::FromRow)]
    struct Row {
        id: i64,
        title: String,
    }

    let rows: Vec<Row> = sqlx::query_as("SELECT id, title FROM audio_groups ORDER BY title")
        .fetch_all(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(rows
        .into_iter()
        .map(|r| MediaTitle {
            id: r.id as u64,
            title: r.title,
        })
        .collect())
}

#[derive(Clone, Copy)]
struct UploadState {
    media_type: RwSignal<Option<MediaType>>,
    is_new: RwSignal<Option<bool>>,
    existing_id: RwSignal<Option<i64>>,
    title: RwSignal<String>,
    description: RwSignal<String>,
    season_number: RwSignal<u32>,
    poster_file: RwSignal<Option<web_sys::File>>,
    items: RwSignal<Vec<UploadItem>>,
    next_id: RwSignal<u32>,
}

impl UploadState {
    fn new() -> Self {
        Self {
            media_type: RwSignal::new(None),
            is_new: RwSignal::new(None),
            existing_id: RwSignal::new(None),
            title: RwSignal::new(String::new()),
            description: RwSignal::new(String::new()),
            season_number: RwSignal::new(1),
            poster_file: RwSignal::new(None),
            items: RwSignal::new(Vec::new()),
            next_id: RwSignal::new(1),
        }
    }

    fn switch_media_type(&self, mt: MediaType) {
        batch(|| {
            self.media_type.set(Some(mt));
            self.is_new.set(None);
            self.existing_id.set(None);
            self.title.set(String::new());
            self.description.set(String::new());
            self.season_number.set(1);
            self.poster_file.set(None);
            self.items.set(Vec::new());
            self.next_id.set(1);
        });
    }
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
                <UploadHeader
                    title="رفع وسائط جديدة"
                    subtitle="أضف فيلماً أو مسلسلاً أو مجموعة صوتية إلى مكتبتك المنزلية"
                    icon=UploadIcon()
                />
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
    let state = UploadState::new();

    // Background job tracking (only used by this component).
    let active_job = RwSignal::new(None::<String>);
    let job_status = RwSignal::new(None::<ConversionStatus>);

    let titles = Resource::new(
        move || state.media_type.get(),
        |mt| async move {
            match mt {
                Some(MediaType::Series) => fetch_series_titles().await,
                Some(MediaType::Movie) => fetch_movie_titles().await,
                Some(MediaType::AudioGroup) => fetch_audio_group_titles().await,
                None => Ok(Vec::new()),
            }
        },
    );

    let s2_ref = NodeRef::<html::Div>::new();
    let s3_ref = NodeRef::<html::Div>::new();

    Effect::new(move |prev: Option<bool>| {
        let curr = state.media_type.get().is_some();
        if !prev.unwrap_or(false) && curr {
            let r = s2_ref;
            set_timeout(
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
        let curr = state.is_new.get().is_some();
        if !prev.unwrap_or(false) && curr {
            let r = s3_ref;
            set_timeout(
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

    // Poll the background job until it reports Done or Failed.
    Effect::new(move |_| {
        let Some(my_id) = active_job.get() else {
            return;
        };
        job_status.set(None);

        leptos::task::spawn_local(async move {
            loop {
                match poll_conversion(my_id.clone()).await {
                    Ok(status) => {
                        let done =
                            matches!(status, ConversionStatus::Done | ConversionStatus::Failed(_));
                        job_status.set(Some(status));
                        if done {
                            break;
                        }
                    }
                    Err(_) => { /* transient; keep trying */ }
                }
                TimeoutFuture::new(700).await;
            }

            if active_job.get_untracked().as_deref() == Some(my_id.as_str()) {
                active_job.set(None);
            }
        });
    });

    // Client-side validation.
    let validation_hint = Signal::derive(move || -> Option<String> {
        let mt = match state.media_type.get() {
            Some(m) => m,
            None => return Some("اختر نوع الوسائط".into()),
        };
        let inw = match state.is_new.get() {
            Some(n) => n,
            None => return Some("اختر إنشاء جديد أو إضافة إلى موجود".into()),
        };
        if !inw && state.existing_id.get().is_none() {
            return Some("اختر العنصر الموجود".into());
        }
        if inw && state.title.get().trim().is_empty() {
            return Some("أدخل عنواناً".into());
        }
        if state.items.get().is_empty() {
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

    Effect::new(move |_| {
        if let Some(Ok(result)) = upload_action.value().get()
            && let Some(id) = result.job_id
        {
            active_job.set(Some(id));
        }
    });

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();

        if validation_hint.get_untracked().is_some() {
            return;
        }

        let snapshot = state.items.get_untracked();

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

        for (i, item) in snapshot.iter().enumerate() {
            let _ = form_data.append_with_blob_and_filename(
                &format!("file_{i}"),
                &item.file,
                &item.file.name(),
            );
            let _ = form_data.append_with_str(&format!("file_title_{i}"), &item.title);
        }

        if let Some(poster) = state.poster_file.get_untracked() {
            let _ = form_data.append_with_blob_and_filename("poster_file", &poster, &poster.name());
        }

        upload_action.dispatch(form_data);
    };

    let result_view = move || match upload_action.value().get() {
        Some(Ok(r)) if r.job_id.is_none() => Some(view! {
            <div class="bg-green-500/15 text-green-300 border border-green-500/30 rounded-xl p-3 text-sm">
                {r.message}
            </div>
        }),
        Some(Ok(_)) => None,
        Some(Err(e)) => Some(view! {
            <div class="bg-red-500/15 text-red-300 border border-red-500/30 rounded-xl p-3 text-sm">
                {e.to_string()}
            </div>
        }),
        None => None,
    };

    view! {
        <form on:submit=on_submit class="space-y-4 md:space-y-5">
            // ① Media type
            <FormSection
                number=1
                title="ما نوع الوسائط؟"
                done=Signal::derive(move || state.media_type.get().is_some())
            >
                <MediaKindCards state/>
            </FormSection>

            // ② New vs existing
            <Show when=move || state.media_type.get().is_some()>
                <div node_ref=s2_ref class="scroll-mt-24 md:scroll-mt-28">
                    <FormSection
                        number=2
                        title="إنشاء جديد أم إضافة إلى موجود؟"
                        done=Signal::derive(move || state.is_new.get().is_some())
                    >
                        <NewOrExistingCards
                            is_new=state.is_new
                            existing_id=state.existing_id
                        />
                    </FormSection>
                </div>
            </Show>

            // ③ Details
            <Show when=move || state.is_new.get().is_some()>
                <div node_ref=s3_ref class="scroll-mt-24 md:scroll-mt-28">
                    <FormSection
                        number=3
                        title="تفاصيل المحتوى"
                        done=Signal::derive(move || {
                            let inw = state.is_new.get();
                            if inw == Some(true) {
                                !state.title.get().trim().is_empty()
                            } else if inw == Some(false) {
                                state.existing_id.get().is_some()
                            } else {
                                false
                            }
                        })
                    >
                        <DetailsSection state titles/>
                    </FormSection>
                </div>
            </Show>

            // ④ Files
            <Show when=move || state.is_new.get().is_some()>
                <FormSection
                    number=4
                    title="الملفات"
                    done=Signal::derive(move || !state.items.get().is_empty())
                >
                    {move || match state.media_type.get() {
                        Some(MediaType::Movie) => view! {
                            <MediaFilesSection
                                items=state.items
                                next_id=state.next_id
                                heading="فصول الفيلم"
                                hint="الفيديوهات بصيغة غير مدعومة (مثل MKV) سيتم تحويلها إلى MP4 تلقائياً."
                                input_id="multiMovieInput"
                                accept=VIDEO_ACCEPT
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
                                items=state.items
                                next_id=state.next_id
                                heading="الحلقات"
                                hint="الفيديوهات بصيغة غير مدعومة (مثل MKV) سيتم تحويلها إلى MP4 تلقائياً."
                                input_id="multiEpisodeInput"
                                accept=VIDEO_ACCEPT
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
                                items=state.items
                                next_id=state.next_id
                                heading="المقاطع الصوتية"
                                hint="الملفات الصوتية بصيغة غير مدعومة (مثل FLAC) سيتم تحويلها إلى MP3 تلقائياً."
                                input_id="multiAudioInput"
                                accept=AUDIO_ACCEPT
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

            <HiddenFormState state/>

            {result_view}

            <Show when=move || job_status.get().is_some()>
                {move || job_status.get().map(|s| view! { <ConversionProgress status=s/> })}
            </Show>

            <UploadSubmitButton
                label="رفع الوسائط".to_string()
                pending=Signal::derive(move || {
                    upload_action.pending().get() || active_job.get().is_some()
                })
                valid=Signal::derive(move || validation_hint.get().is_none())
                hint=Signal::derive(move || validation_hint.get().unwrap_or_default())
                file_count=Signal::derive(move || state.items.get().len())
            />
        </form>
    }
}

// ─── Section ① ────────────────────────────────────────────────────────────

#[component]
fn MediaKindCards(state: UploadState) -> impl IntoView {
    let on_select: Rc<dyn Fn(MediaType)> = Rc::new(move |mt: MediaType| {
        if state.media_type.get_untracked() == Some(mt) {
            return;
        }
        if !state.items.get_untracked().is_empty() {
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
        state.switch_media_type(mt);
    });

    view! {
        <div class="grid grid-cols-3 gap-3">
            <MediaKindCard value=MediaType::Movie label="فيلم" icon=MovieIcon()
                media_type=state.media_type on_select=Rc::clone(&on_select)/>
            <MediaKindCard value=MediaType::Series label="مسلسل" icon=SeriesIcon()
                media_type=state.media_type on_select=Rc::clone(&on_select)/>
            <MediaKindCard value=MediaType::AudioGroup label="صوتيات" icon=AudioIcon()
                media_type=state.media_type on_select=on_select/>
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

// ─── Section ② ────────────────────────────────────────────────────────────

#[component]
fn NewOrExistingCards(
    is_new: RwSignal<Option<bool>>,
    existing_id: RwSignal<Option<i64>>,
) -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <ChoiceCard
                active=Signal::derive(move || is_new.get() == Some(true))
                label="إنشاء جديد" sub="ابدأ من الصفر"
                on_click=move |_: MouseEvent| {
                    is_new.set(Some(true));
                    existing_id.set(None);
                }
            />
            <ChoiceCard
                active=Signal::derive(move || is_new.get() == Some(false))
                label="إضافة إلى موجود" sub="أضف إلى عنصر موجود في المكتبة"
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

// ─── Section ③ ────────────────────────────────────────────────────────────

#[component]
fn DetailsSection(
    state: UploadState,
    titles: Resource<Result<Vec<MediaTitle>, ServerFnError>>,
) -> impl IntoView {
    let is_new_true = Memo::new(move |_| state.is_new.get() == Some(true));
    let is_new_false = Memo::new(move |_| state.is_new.get() == Some(false));
    let is_series = Memo::new(move |_| state.media_type.get() == Some(MediaType::Series));

    view! {
        <div class="space-y-4">
            <Show when=move || is_new_true.get()>
                <TitleField title=state.title/>
                <DescriptionField description=state.description/>
            </Show>

            <Show when=move || is_new_false.get()>
                <ExistingSelectField state titles/>
            </Show>

            <PosterUploadField poster_file=state.poster_file/>

            <Show when=move || is_series.get()>
                <SeasonNumberField season_number=state.season_number/>
            </Show>
        </div>
    }
}

#[component]
fn ExistingSelectField(
    state: UploadState,
    titles: Resource<Result<Vec<MediaTitle>, ServerFnError>>,
) -> impl IntoView {
    let label = match state.media_type.get() {
        Some(MediaType::Series) => "اختر المسلسل الموجود",
        Some(MediaType::Movie) => "اختر الفيلم الموجود",
        Some(MediaType::AudioGroup) => "اختر المجموعة الصوتية الموجودة",
        None => "اختر العنصر",
    };
    let adapter = move |list: Vec<MediaTitle>| ExistingSelectInnerProps {
        existing_id: state.existing_id,
        list,
    };
    view! {
        <div>
            <label class="block text-sm font-medium text-gray-300 mb-1.5">{label}</label>
            <ResourceView resource=titles view_fn=ExistingSelectInner adapter=adapter/>
        </div>
    }
}

#[component]
fn ExistingSelectInner(existing_id: RwSignal<Option<i64>>, list: Vec<MediaTitle>) -> impl IntoView {
    view! {
        <select
            on:change=move |ev| {
                if let Some(sel) = ev.target().and_then(|t| t.dyn_into::<HtmlSelectElement>().ok()) {
                    existing_id.set(sel.value().parse().ok());
                }
            }
            class=INPUT_CLASS
        >
            <option value="" class="bg-gray-800">"-- اختر --"</option>
            {list.into_iter().map(|item| view! {
                <option value={item.id.to_string()} class="bg-gray-800">{item.title}</option>
            }).collect_view()}
        </select>
    }
}

// ─── Hidden form state ────────────────────────────────────────────────────

#[component]
fn HiddenFormState(state: UploadState) -> impl IntoView {
    view! {
        <input type="hidden" name="media_type"
            value=move || state.media_type.get().map(|m| m.to_string()).unwrap_or_default()/>
        <input type="hidden" name="is_new"
            value=move || state.is_new.get().map(|b| b.to_string()).unwrap_or_default()/>
        <input type="hidden" name="existing_id"
            value=move || state.existing_id.get().map(|id| id.to_string()).unwrap_or_default()/>
    }
}
