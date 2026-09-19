use crate::app::{
    icons::{AudioIcon, MovieIcon, SeriesIcon},
    model::MediaType,
    route_params::use_u64_param,
    upload_api::{poll_conversion, upload_media},
    upload_ui::{
        AUDIO_ACCEPT, ConversionProgress, ConversionStatus, DescriptionField, FormSection,
        MediaFilesSection, PosterUploadField, SeasonNumberField, TitleField, UploadHeader,
        UploadItem, UploadSubmitButton, VIDEO_ACCEPT,
    },
};
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use leptos_router::{LazyRoute, lazy_route};
use web_sys::{FormData, HtmlFormElement, wasm_bindgen::JsCast};

#[derive(Clone, Copy)]
struct NewUploadState {
    media_type: MediaType,
    title: RwSignal<String>,
    description: RwSignal<String>,
    season_number: RwSignal<u32>,
    poster_file: RwSignal<Option<web_sys::File>>,
    items: RwSignal<Vec<UploadItem>>,
    next_id: RwSignal<u32>,
}

impl NewUploadState {
    fn new(media_type: MediaType) -> Self {
        Self {
            media_type,
            title: RwSignal::new(String::new()),
            description: RwSignal::new(String::new()),
            season_number: RwSignal::new(1),
            poster_file: RwSignal::new(None),
            items: RwSignal::new(Vec::new()),
            next_id: RwSignal::new(1),
        }
    }
}

// ─── Route wrappers ───────────────────────────────────────────────────────

pub struct NewMoviePage;

#[lazy_route]
impl LazyRoute for NewMoviePage {
    fn data() -> Self {
        Self
    }
    fn view(_this: Self) -> AnyView {
        view! { <NewUploadForm media_type=MediaType::Movie/> }.into_any()
    }
}

pub struct NewSeriesPage;

#[lazy_route]
impl LazyRoute for NewSeriesPage {
    fn data() -> Self {
        Self
    }
    fn view(_this: Self) -> AnyView {
        view! { <NewUploadForm media_type=MediaType::Series/> }.into_any()
    }
}

pub struct NewAudioGroupPage;

#[lazy_route]
impl LazyRoute for NewAudioGroupPage {
    fn data() -> Self {
        Self
    }
    fn view(_this: Self) -> AnyView {
        view! { <NewUploadForm media_type=MediaType::AudioGroup/> }.into_any()
    }
}

// ─── Shared form ──────────────────────────────────────────────────────────

#[component]
fn NewUploadForm(media_type: MediaType) -> impl IntoView {
    let state = NewUploadState::new(media_type);

    let active_job = RwSignal::new(None::<String>);
    let job_status = RwSignal::new(None::<ConversionStatus>);

    let upload_action = Action::new_local(|data: &FormData| upload_media(data.clone().into()));

    // Poll a background conversion job until it reports terminal state.
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
                    Err(_) => { /* transient */ }
                }
                TimeoutFuture::new(700).await;
            }

            if active_job.get_untracked().as_deref() == Some(my_id.as_str()) {
                active_job.set(None);
            }
        });
    });

    // Pick up the job id from the server's response.
    Effect::new(move |_| {
        if let Some(Ok(result)) = upload_action.value().get()
            && let Some(id) = result.job_id
        {
            active_job.set(Some(id));
        }
    });

    // On success, bounce to the listing. Fast path succeeds with no job id;
    // slow path reaches `ConversionStatus::Done`.
    let navigate = use_navigate();
    let listing_href = media_type.listing_href();
    let listing_href_for_fast = listing_href.clone();
    let listing_href_for_slow = listing_href.clone();

    Effect::new({
        let nav = navigate.clone();
        move |_| {
            if let Some(Ok(r)) = upload_action.value().get()
                && r.job_id.is_none()
            {
                nav(&listing_href_for_fast, Default::default());
            }
        }
    });

    Effect::new({
        let nav = navigate.clone();
        move || {
            if matches!(job_status.get(), Some(ConversionStatus::Done)) {
                nav(&listing_href_for_slow, Default::default());
            }
        }
    });

    // Validation.
    let validation_hint = Signal::derive(move || -> Option<String> {
        if state.title.get().trim().is_empty() {
            return Some("أدخل عنواناً".into());
        }
        if state.items.get().is_empty() {
            return Some(
                match state.media_type {
                    MediaType::Movie => "أضف فصلاً واحداً على الأقل",
                    MediaType::Series => "أضف حلقة واحدة على الأقل",
                    MediaType::AudioGroup => "أضف مقطعاً صوتياً واحداً على الأقل",
                }
                .into(),
            );
        }
        None
    });

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if validation_hint.get_untracked().is_some() {
            return;
        }

        let snapshot = state.items.get_untracked();

        let Some(form) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlFormElement>().ok())
        else {
            return;
        };

        let Ok(form_data) = FormData::new_with_form(&form) else {
            return;
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
        Some(Err(e)) => Some(view! {
            <div class="bg-red-500/15 text-red-300 border border-red-500/30 rounded-xl p-3 text-sm">
                {e.to_string()}
            </div>
        }),
        Some(Ok(_)) => None, // success navigates away
        None => None,
    };

    let header_title = match media_type {
        MediaType::Movie => "رفع فيلم جديد",
        MediaType::Series => "رفع مسلسل جديد",
        MediaType::AudioGroup => "رفع مجموعة صوتية جديدة",
    };
    let header_subtitle = match media_type {
        MediaType::Movie => "أضف فيلماً إلى مكتبتك المنزلية",
        MediaType::Series => "أضف مسلسلاً إلى مكتبتك المنزلية",
        MediaType::AudioGroup => "أضف مجموعة صوتية إلى مكتبتك المنزلية",
    };
    let header_icon = match media_type {
        MediaType::Movie => MovieIcon().into_any(),
        MediaType::Series => SeriesIcon().into_any(),
        MediaType::AudioGroup => AudioIcon().into_any(),
    };
    let submit_label = match media_type {
        MediaType::Movie => "رفع الفيلم",
        MediaType::Series => "رفع المسلسل",
        MediaType::AudioGroup => "رفع المجموعة",
    };

    view! {
        <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
            <UploadHeader title=header_title subtitle=header_subtitle icon=header_icon/>
            <div class=crate::app::upload_ui::CARD_CLASS>
                <form on:submit=on_submit class="space-y-4 md:space-y-5">
                    <FormSection
                        number=1
                        title="تفاصيل المحتوى"
                        done=Signal::derive(move || !state.title.get().trim().is_empty())
                    >
                        <div class="space-y-4">
                            <TitleField title=state.title/>
                            <DescriptionField description=state.description/>
                            <PosterUploadField poster_file=state.poster_file/>
                            <Show when=move || media_type == MediaType::Series>
                                <SeasonNumberField season_number=state.season_number/>
                            </Show>
                        </div>
                    </FormSection>

                    // ② Files
                    <FormSection
                        number=2
                        title="الملفات"
                        done=Signal::derive(move || !state.items.get().is_empty())
                    >
                        {move || match media_type {
                            MediaType::Movie => view! {
                                <MediaFilesSection
                                    items=state.items
                                    next_id=state.next_id
                                    heading="فصول الفيلم"
                                    hint="الفيديوهات بصيغة غير مدعومة (مثل MKV) سيتم تحويلها إلى MP4 تلقائياً."
                                    input_id="newMovieInput"
                                    accept=VIDEO_ACCEPT
                                    select_label="اختيار فصول الفيلم"
                                    number_label="رقم الفصل"
                                    title_label="عنوان الفصل"
                                    file_label="الملف"
                                    icon=MovieIcon()
                                />
                            }
                            .into_any(),
                            MediaType::Series => view! {
                                <MediaFilesSection
                                    items=state.items
                                    next_id=state.next_id
                                    heading="الحلقات"
                                    hint="الفيديوهات بصيغة غير مدعومة (مثل MKV) سيتم تحويلها إلى MP4 تلقائياً."
                                    input_id="newSeriesInput"
                                    accept=VIDEO_ACCEPT
                                    select_label="اختيار الحلقات"
                                    number_label="رقم الحلقة"
                                    title_label="عنوان الحلقة"
                                    file_label="الملف"
                                    icon=SeriesIcon()
                                />
                            }
                            .into_any(),
                            MediaType::AudioGroup => view! {
                                <MediaFilesSection
                                    items=state.items
                                    next_id=state.next_id
                                    heading="المقاطع الصوتية"
                                    hint="الملفات الصوتية بصيغة غير مدعومة (مثل FLAC) سيتم تحويلها إلى MP3 تلقائياً."
                                    input_id="newAudioInput"
                                    accept=AUDIO_ACCEPT
                                    select_label="اختيار ملفات صوتية"
                                    number_label="رقم المقطع"
                                    title_label="عنوان المقطع الصوتي"
                                    file_label="الملف"
                                    icon=AudioIcon()
                                />
                            }
                            .into_any(),
                        }}
                    </FormSection>

                    // Hidden wire-format fields. `is_new` is always true and
                    // `existing_id` always empty — the URL encodes the rest.
                    <input type="hidden" name="media_type" value=media_type.to_string()/>
                    <input type="hidden" name="is_new" value="true"/>
                    <input type="hidden" name="existing_id" value=""/>
                    <input
                        type="hidden"
                        name="season_number"
                        value=move || state.season_number.get().to_string()
                    />

                    {result_view}

                    <Show when=move || job_status.get().is_some()>
                        {move || job_status.get().map(|s| view! { <ConversionProgress status=s/> })}
                    </Show>

                    <UploadSubmitButton
                        label=submit_label.to_string()
                        pending=Signal::derive(move || {
                            upload_action.pending().get() || active_job.get().is_some()
                        })
                        valid=Signal::derive(move || validation_hint.get().is_none())
                        hint=Signal::derive(move || validation_hint.get().unwrap_or_default())
                        file_count=Signal::derive(move || state.items.get().len())
                    />
                </form>
            </div>
        </div>
    }
}

// ─── Append form ──────────────────────────────────────────────────────────

#[derive(Clone, Copy)]
struct AppendUploadState {
    media_type: MediaType,
    target_id: u64,
    season_number: RwSignal<u32>,
    items: RwSignal<Vec<UploadItem>>,
    next_id: RwSignal<u32>,
}

impl AppendUploadState {
    fn new(media_type: MediaType, target_id: u64) -> Self {
        Self {
            media_type,
            target_id,
            season_number: RwSignal::new(1),
            items: RwSignal::new(Vec::new()),
            next_id: RwSignal::new(1),
        }
    }
}

#[component]
fn AppendUploadForm(media_type: MediaType, target_id: u64) -> impl IntoView {
    let state = AppendUploadState::new(media_type, target_id);

    let active_job = RwSignal::new(None::<String>);
    let job_status = RwSignal::new(None::<ConversionStatus>);

    let upload_action = Action::new_local(|data: &FormData| upload_media(data.clone().into()));

    // Poll the background job until it reaches a terminal state.
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
                    Err(_) => { /* transient */ }
                }
                TimeoutFuture::new(700).await;
            }
            if active_job.get_untracked().as_deref() == Some(my_id.as_str()) {
                active_job.set(None);
            }
        });
    });

    // Adopt the job id from the server response.
    Effect::new(move |_| {
        if let Some(Ok(result)) = upload_action.value().get()
            && let Some(id) = result.job_id
        {
            active_job.set(Some(id));
        }
    });

    // Navigate back to the detail page on success — both paths.
    let navigate = use_navigate();
    let back_href = media_type.detail_href(target_id);
    let back_href_a = back_href.clone();
    let back_href_b = back_href.clone();

    Effect::new({
        let navigate = navigate.clone();
        move || {
            if let Some(Ok(r)) = upload_action.value().get()
                && r.job_id.is_none()
            {
                navigate(&back_href_a, Default::default());
            }
        }
    });

    Effect::new({
        let navigate = navigate.clone();
        move || {
            if matches!(job_status.get(), Some(ConversionStatus::Done)) {
                navigate(&back_href_b, Default::default());
            }
        }
    });

    // Validation: at least one file; series also needs a valid season number.
    let validation_hint = Signal::derive(move || -> Option<String> {
        if state.items.get().is_empty() {
            return Some(
                match state.media_type {
                    MediaType::Movie => "أضف فصلاً واحداً على الأقل",
                    MediaType::Series => "أضف حلقة واحدة على الأقل",
                    MediaType::AudioGroup => "أضف مقطعاً صوتياً واحداً على الأقل",
                }
                .into(),
            );
        }
        if state.media_type == MediaType::Series && state.season_number.get() < 1 {
            return Some("أدخل رقم موسم صحيح".into());
        }
        None
    });

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if validation_hint.get_untracked().is_some() {
            return;
        }

        let snapshot = state.items.get_untracked();

        let Some(form) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlFormElement>().ok())
        else {
            return;
        };
        let Ok(form_data) = FormData::new_with_form(&form) else {
            return;
        };

        // `title` and `description` are irrelevant for append — the server
        // only reads them on `is_new=true`. Send empty values.
        let _ = form_data.append_with_str("title", "");
        let _ = form_data.append_with_str("description", "");
        let _ = form_data.append_with_str("media_type", &media_type.to_string());
        let _ = form_data.append_with_str("is_new", "false");
        let _ = form_data.append_with_str("existing_id", &target_id.to_string());
        let _ = form_data.append_with_str(
            "season_number",
            &state.season_number.get_untracked().to_string(),
        );

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
        Some(Err(e)) => Some(view! {
            <div class="bg-red-500/15 text-red-300 border border-red-500/30 rounded-xl p-3 text-sm">
                {e.to_string()}
            </div>
        }),
        _ => None,
    };

    let kind_icon = match media_type {
        MediaType::Movie => MovieIcon().into_any(),
        MediaType::Series => SeriesIcon().into_any(),
        MediaType::AudioGroup => AudioIcon().into_any(),
    };
    let header_title = match media_type {
        MediaType::Movie => "إضافة فصول",
        MediaType::Series => "إضافة حلقات",
        MediaType::AudioGroup => "إضافة مقاطع صوتية",
    };
    let header_subtitle = match media_type {
        MediaType::Movie => "سيتم إضافة الفصول إلى هذا الفيلم",
        MediaType::Series => "سيتم إضافة الحلقات إلى الموسم المحدد",
        MediaType::AudioGroup => "سيتم إضافة المقاطع إلى هذه المجموعة",
    };
    let submit_label = match media_type {
        MediaType::Movie => "إضافة الفصول",
        MediaType::Series => "إضافة الحلقات",
        MediaType::AudioGroup => "إضافة المقاطع",
    };

    view! {
        <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
            <UploadHeader title=header_title subtitle=header_subtitle icon=kind_icon/>
            <div class=crate::app::upload_ui::CARD_CLASS>
                <form on:submit=on_submit class="space-y-4 md:space-y-5">
                    // For series, ask which season before the files section.
                    <Show when=move || media_type == MediaType::Series>
                        <FormSection
                            number=1
                            title="الموسم"
                            done=Signal::derive(move || state.season_number.get() >= 1)
                        >
                            <div class="space-y-3">
                                <SeasonNumberField season_number=state.season_number/>
                                <p class="text-xs text-gray-500">
                                    "إذا كان الموسم غير موجود، سيتم إنشاؤه تلقائياً."
                                </p>
                            </div>
                        </FormSection>
                    </Show>

                    <FormSection
                        number={if media_type == MediaType::Series { 2 } else { 1 }}
                        title="الملفات"
                        done=Signal::derive(move || !state.items.get().is_empty())
                    >
                        {move || match media_type {
                            MediaType::Movie => view! {
                                <MediaFilesSection
                                    items=state.items
                                    next_id=state.next_id
                                    heading="فصول الفيلم"
                                    hint="الفيديوهات بصيغة غير مدعومة (مثل MKV) سيتم تحويلها إلى MP4 تلقائياً."
                                    input_id="appendMovieInput"
                                    accept=VIDEO_ACCEPT
                                    select_label="اختيار فصول"
                                    number_label="رقم الفصل"
                                    title_label="عنوان الفصل"
                                    file_label="الملف"
                                    icon=MovieIcon()
                                />
                            }
                            .into_any(),
                            MediaType::Series => view! {
                                <MediaFilesSection
                                    items=state.items
                                    next_id=state.next_id
                                    heading="حلقات الموسم"
                                    hint="الفيديوهات بصيغة غير مدعومة (مثل MKV) سيتم تحويلها إلى MP4 تلقائياً."
                                    input_id="appendSeriesInput"
                                    accept=VIDEO_ACCEPT
                                    select_label="اختيار حلقات"
                                    number_label="رقم الحلقة"
                                    title_label="عنوان الحلقة"
                                    file_label="الملف"
                                    icon=SeriesIcon()
                                />
                            }
                            .into_any(),
                            MediaType::AudioGroup => view! {
                                <MediaFilesSection
                                    items=state.items
                                    next_id=state.next_id
                                    heading="المقاطع الصوتية"
                                    hint="الملفات الصوتية بصيغة غير مدعومة (مثل FLAC) سيتم تحويلها إلى MP3 تلقائياً."
                                    input_id="appendAudioInput"
                                    accept=AUDIO_ACCEPT
                                    select_label="اختيار ملفات صوتية"
                                    number_label="رقم المقطع"
                                    title_label="عنوان المقطع الصوتي"
                                    file_label="الملف"
                                    icon=AudioIcon()
                                />
                            }
                            .into_any(),
                        }}
                    </FormSection>

                    {result_view}

                    <Show when=move || job_status.get().is_some()>
                        {move || job_status.get().map(|s| view! { <ConversionProgress status=s/> })}
                    </Show>

                    <UploadSubmitButton
                        label=submit_label.to_string()
                        pending=Signal::derive(move || {
                            upload_action.pending().get() || active_job.get().is_some()
                        })
                        valid=Signal::derive(move || validation_hint.get().is_none())
                        hint=Signal::derive(move || validation_hint.get().unwrap_or_default())
                        file_count=Signal::derive(move || state.items.get().len())
                    />
                </form>
            </div>
        </div>
    }
}

// ─── Append route wrappers ────────────────────────────────────────────────

pub struct MovieAppendPage;
pub struct SeriesAppendPage;
pub struct AudioGroupAppendPage;

macro_rules! append_page {
    ($page:ident, $kind:expr) => {
        #[lazy_route]
        impl LazyRoute for $page {
            fn data() -> Self {
                Self
            }
            fn view(_this: Self) -> AnyView {
                view! {
                    <AppendUploadForm media_type=$kind target_id=use_u64_param("id")()/>
                }
                .into_any()
            }
        }
    };
}

append_page!(MovieAppendPage, MediaType::Movie);
append_page!(SeriesAppendPage, MediaType::Series);
append_page!(AudioGroupAppendPage, MediaType::AudioGroup);
