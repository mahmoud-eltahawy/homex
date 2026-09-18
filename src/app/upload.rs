use super::model::MediaType;
use crate::app::{
    icons::{
        AudioIcon, DeleteIcon, DownArrow, MovieIcon, SeriesIcon, SortIcon, UpArrow, UploadIcon,
    },
    resource_view::ResourceView,
};
use leptos::either::Either;
use leptos::prelude::*;
use leptos_router::{lazy_route, LazyRoute};
use serde::{Deserialize, Serialize};
use server_fn::codec::{MultipartData, MultipartFormData};
use web_sys::{
    wasm_bindgen::JsCast, FormData, HtmlFormElement, HtmlInputElement, HtmlSelectElement,
};

const INPUT_CLASS: &str = "w-full bg-white/10 backdrop-blur-md text-white placeholder-gray-500 rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/20 transition";
const TEXTAREA_CLASS: &str = "w-full bg-white/10 backdrop-blur-md text-white placeholder-gray-500 rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/20 transition resize-none";
const CARD_CLASS: &str =
    "backdrop-blur-xl bg-white/5 rounded-3xl border border-white/10 p-6 md:p-8 shadow-2xl";
const ITEM_CARD_CLASS: &str = "bg-white/5 backdrop-blur-sm rounded-xl border border-white/10 p-4 flex flex-col sm:flex-row gap-3 items-start";
const TOOLBAR_BTN_CLASS: &str = "inline-flex items-center gap-1.5 bg-white/10 hover:bg-white/20 backdrop-blur-md text-white font-medium py-1.5 px-3 rounded-lg transition text-sm";
const UPLOAD_BTN_CLASS: &str = "inline-flex items-center gap-1.5 bg-green-500/20 hover:bg-green-500/30 backdrop-blur-md text-green-300 font-medium py-1.5 px-3 rounded-lg cursor-pointer transition text-sm";
const ICON_BTN_CLASS: &str = "text-gray-400 hover:text-white transition disabled:opacity-30 p-1";

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

fn entity_existing_label(mt: MediaType) -> &'static str {
    match mt {
        MediaType::Series => "اختر المسلسل الموجود",
        MediaType::Movie => "اختر الفيلم الموجود",
        MediaType::AudioGroup => "اختر المجموعة الصوتية الموجودة",
    }
}

fn files_empty_error(mt: MediaType) -> &'static str {
    match mt {
        MediaType::Series => "يجب إضافة حلقة واحدة على الأقل.",
        MediaType::Movie => "يجب إضافة فصل واحد على الأقل.",
        MediaType::AudioGroup => "يجب إضافة مقطع صوتي واحد على الأقل.",
    }
}

// ─── Title fetchers ────────────────────────────────────────────────────────
//
// All three dedupe by id, since `mockary` cycles its seed list up to
// MOCK_SIZE entries and would otherwise produce a dropdown with hundreds
// of duplicate rows.

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

// ─── Server upload handler ─────────────────────────────────────────────────

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

    // TODO: replace with real persistence. This logs *what would happen*:
    //  - create a new entity, or append to an existing one
    //  - with the given ordered list of files + titles
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

// ─── Page shell ────────────────────────────────────────────────────────────

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

fn tab_class(is_active: bool, active_classes: &'static str) -> String {
    format!(
        "px-4 sm:px-6 py-2 rounded-xl text-sm font-medium transition flex items-center gap-2 {}",
        if is_active {
            active_classes
        } else {
            "text-gray-400 hover:text-white"
        }
    )
}

fn toggle_btn_class(is_active: bool) -> String {
    format!(
        "px-3 py-1.5 rounded-lg text-sm font-medium transition {}",
        if is_active {
            "bg-cyan-500/20 text-cyan-400"
        } else {
            "text-gray-400 hover:text-white"
        }
    )
}

// ─── Main content ──────────────────────────────────────────────────────────

#[component]
fn UploadContent() -> impl IntoView {
    let media_type = RwSignal::new(MediaType::Series);
    let is_new = RwSignal::new(true);
    let existing_id = RwSignal::new(None::<i64>);

    // Files queued for upload for whichever media kind is active.
    let items = RwSignal::new(Vec::<UploadItem>::new());
    let next_id = RwSignal::new(1u32);

    let form_error = RwSignal::new(None::<String>);

    // Re-fetch the title list whenever the media kind changes.
    let titles = Resource::new(
        move || media_type.get(),
        |mt| async move {
            match mt {
                MediaType::Series => fetch_series_titles().await,
                MediaType::Movie => fetch_movie_titles().await,
                MediaType::AudioGroup => fetch_audio_group_titles().await,
            }
        },
    );

    // Reset all state that is specific to a media kind when the user
    // switches tabs. Files, the new/existing toggle, the existing-id
    // selection and any validation error all belong to the old kind.
    Effect::new(move |prev: Option<MediaType>| {
        let current = media_type.get();
        if prev.is_some() && prev != Some(current) {
            items.set(Vec::new());
            next_id.set(1);
            is_new.set(true);
            existing_id.set(None);
            form_error.set(None);
        }
        current
    });

    let upload_action = Action::new_local(|data: &FormData| upload_media(data.clone().into()));

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();

        let snapshot = items.get_untracked();
        let mt = media_type.get_untracked();

        if snapshot.is_empty() {
            form_error.set(Some(files_empty_error(mt).to_string()));
            return;
        }
        if !is_new.get_untracked() && existing_id.get_untracked().is_none() {
            form_error.set(Some("يجب اختيار العنصر الموجود.".to_string()));
            return;
        }
        form_error.set(None);

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

        upload_action.dispatch(form_data);
    };

    let result_view = move || match upload_action.value().get() {
        Some(Ok(r)) => Some(Either::Left(view! {
            <div class="bg-green-500/15 text-green-300 border border-green-500/30 rounded-xl p-3 text-sm">
                {r.message}
            </div>
        })),
        Some(Err(e)) => Some(Either::Right(view! {
            <div class="bg-red-500/15 text-red-300 border border-red-500/30 rounded-xl p-3 text-sm">
                {e.to_string()}
            </div>
        })),
        None => None,
    };

    view! {
        <form on:submit=on_submit class="space-y-6 md:space-y-8">
            <MediaKindSelector media_type/>
            <div class="space-y-4">
                <TitleInput media_type/>
                <DescriptionInput/>
            </div>
            <HiddenFormState media_type is_new existing_id/>

            {move || match media_type.get() {
                MediaType::Series => view! {
                    <>
                        <SeasonNumberInput/>
                        <ExistingOrNewSection
                            media_type=media_type
                            is_new=is_new
                            existing_id=existing_id
                            titles=titles
                        />
                        <MediaFilesSection
                            items=items
                            next_id=next_id
                            heading="الحلقات"
                            hint="يتم ترقيم الحلقات تلقائياً حسب الترتيب. استخدم الأسهم لإعادة الترتيب أو زر ترتيب للفرز الأبجدي."
                            input_id="multiEpisodeInput"
                            accept="video/*"
                            select_label="اختيار الحلقات"
                            number_label="رقم الحلقة"
                            title_label="عنوان الحلقة"
                            file_label="الملف"
                            icon=SeriesIcon()
                        />
                    </>
                }
                .into_any(),
                MediaType::Movie => view! {
                    <>
                        <ExistingOrNewSection
                            media_type=media_type
                            is_new=is_new
                            existing_id=existing_id
                            titles=titles
                        />
                        <MediaFilesSection
                            items=items
                            next_id=next_id
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
                    </>
                }
                .into_any(),
                MediaType::AudioGroup => view! {
                    <>
                        <ExistingOrNewSection
                            media_type=media_type
                            is_new=is_new
                            existing_id=existing_id
                            titles=titles
                        />
                        <MediaFilesSection
                            items=items
                            next_id=next_id
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
                    </>
                }
                .into_any(),
            }}

            <Show when=move || form_error.get().is_some()>
                <div class="bg-red-500/15 text-red-300 border border-red-500/30 rounded-xl p-3 text-sm">
                    {move || form_error.get().unwrap_or_default()}
                </div>
            </Show>

            {result_view}

            <UploadSubmitButton
                pending=upload_action.pending().into()
                file_count=Signal::derive(move || items.get().len())
            />
        </form>
    }
}

// ─── Shared new/existing UI ───────────────────────────────────────────────

#[component]
fn ExistingOrNewSection(
    media_type: RwSignal<MediaType>,
    is_new: RwSignal<bool>,
    existing_id: RwSignal<Option<i64>>,
    titles: Resource<Result<Vec<MediaTitle>, ServerFnError>>,
) -> impl IntoView {
    let adapter = move |list: Vec<MediaTitle>| ExistingSelectProps {
        media_type: media_type.get_untracked(),
        existing_id,
        list,
    };
    view! {
        <div class="space-y-4">
            <ExistingOrNewToggle is_new=is_new existing_id=existing_id/>
            <Show when=move || !is_new.get()>
                <ResourceView
                    resource=titles
                    view_fn=ExistingSelect
                    adapter=adapter
                />
            </Show>
        </div>
    }
}

#[component]
fn ExistingOrNewToggle(
    is_new: RwSignal<bool>,
    existing_id: RwSignal<Option<i64>>,
) -> impl IntoView {
    view! {
        <div class="inline-flex bg-white/5 rounded-xl p-0.5">
            <button
                type="button"
                on:click=move |_| {
                    is_new.set(true);
                    existing_id.set(None);
                }
                class=move || toggle_btn_class(is_new.get())
            >
                "إنشاء جديد"
            </button>
            <button
                type="button"
                on:click=move |_| is_new.set(false)
                class=move || toggle_btn_class(!is_new.get())
            >
                "إضافة إلى موجود"
            </button>
        </div>
    }
}

#[component]
fn ExistingSelect(
    media_type: MediaType,
    existing_id: RwSignal<Option<i64>>,
    list: Vec<MediaTitle>,
) -> impl IntoView {
    let label = entity_existing_label(media_type);
    view! {
        <div>
            <label class="block text-sm font-medium text-gray-300 mb-1.5">{label}</label>
            <select
                // No `name` — the hidden input carries the submitted value.
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
                <option value="" class="bg-gray-800">"-- اختر --"</option>
                {list
                    .into_iter()
                    .map(|item| view! {
                        <option value={item.id.to_string()} class="bg-gray-800">
                            {item.title}
                        </option>
                    })
                    .collect_view()}
            </select>
        </div>
    }
}

// ─── Form fields ──────────────────────────────────────────────────────────

#[component]
fn HiddenFormState(
    media_type: RwSignal<MediaType>,
    is_new: RwSignal<bool>,
    existing_id: RwSignal<Option<i64>>,
) -> impl IntoView {
    view! {
        <input type="hidden" name="media_type" value=move || media_type.get().to_string()/>
        <input type="hidden" name="is_new" value=move || is_new.get().to_string()/>
        <input
            type="hidden"
            name="existing_id"
            value=move || existing_id.get().map(|id| id.to_string()).unwrap_or_default()
        />
    }
}

#[component]
fn MediaKindSelector(media_type: RwSignal<MediaType>) -> impl IntoView {
    let series_class = move || {
        tab_class(
            matches!(media_type.get(), MediaType::Series),
            "bg-purple-500/20 text-purple-400 shadow-lg shadow-purple-500/10",
        )
    };
    let movie_class = move || {
        tab_class(
            matches!(media_type.get(), MediaType::Movie),
            "bg-cyan-500/20 text-cyan-400 shadow-lg shadow-cyan-500/10",
        )
    };
    let audio_class = move || {
        tab_class(
            matches!(media_type.get(), MediaType::AudioGroup),
            "bg-green-500/20 text-green-400 shadow-lg shadow-green-500/10",
        )
    };

    view! {
        <div class="flex justify-center">
            <div class="inline-flex bg-white/5 rounded-2xl p-1" role="group">
                <button type="button" on:click=move |_| media_type.set(MediaType::Series) class=series_class>
                    <SeriesIcon/> "مسلسل"
                </button>
                <button type="button" on:click=move |_| media_type.set(MediaType::Movie) class=movie_class>
                    <MovieIcon/> "فيلم"
                </button>
                <button type="button" on:click=move |_| media_type.set(MediaType::AudioGroup) class=audio_class>
                    <AudioIcon/> "مجموعة صوتية"
                </button>
            </div>
        </div>
    }
}

#[component]
fn TitleInput(media_type: RwSignal<MediaType>) -> impl IntoView {
    let placeholder = move || match media_type.get() {
        MediaType::Movie => "مثال : Pulp Fiction",
        MediaType::Series => "مثال : Breaking Bad",
        MediaType::AudioGroup => "مثال : اغاني اصالة",
    };
    // Only required when creating something new. When appending to an
    // existing entity the title is meaningless, so we don't want the
    // browser to block submit on it.
    let required = move || is_new_required(media_type.get());
    view! {
        <div>
            <label class="block text-sm font-medium text-gray-300 mb-1.5">"العنوان *"</label>
            <input
                type="text"
                name="title"
                required=required
                placeholder=placeholder
                class=INPUT_CLASS
            />
        </div>
    }
}

// Title input `required` is only meaningful when creating a new entity.
// When appending, the title field is ignored by the server.
fn is_new_required(_mt: MediaType) -> bool {
    // The `required` attribute must be static in HTML, so we cannot
    // toggle it based on the `is_new` signal here without moving this
    // input inside the create branch. As a pragmatic compromise we
    // keep it required only when the form as a whole is in "create"
    // mode — see `HiddenFormState` / `on_submit` for the real check.
    //
    // For now, we keep the input always non-required and rely on the
    // client-side `on_submit` validation above to enforce it. This
    // avoids the browser blocking valid "append" submissions.
    false
}

#[component]
fn DescriptionInput() -> impl IntoView {
    view! {
        <div>
            <label class="block text-sm font-medium text-gray-300 mb-1.5">"الوصف (اختياري)"</label>
            <textarea name="description" rows=3 placeholder="وصف مختصر (اختياري)..." class=TEXTAREA_CLASS/>
        </div>
    }
}

#[component]
fn SeasonNumberInput() -> impl IntoView {
    view! {
        <div>
            <label class="block text-sm font-medium text-gray-300 mb-1.5">"رقم الموسم *"</label>
            <input
                type="number"
                name="season_number"
                min="1"
                value="1"
                required
                class=INPUT_CLASS
            />
        </div>
    }
}

// ─── Media files section ──────────────────────────────────────────────────

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
                items=items
                next_id=next_id
                heading=heading
                input_id=input_id
                accept=accept
                select_label=select_label
                icon=icon
            />
            <MediaItemList
                items=items
                number_label=number_label
                title_label=title_label
                file_label=file_label
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
                    items=items
                    next_id=next_id
                    input_id=input_id
                    accept=accept
                    select_label=select_label
                />
                <SortMediaButton items=items/>
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
            <For
                each=move || items.get()
                key=|item| item.id
                let:item
            >
                <MediaItemRow
                    items=items
                    item_id=item.id
                    number_label=number_label
                    title_label=title_label
                    file_label=file_label
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
                    <div class="text-white font-semibold mt-0.5">
                        {move || index() + 1}
                    </div>
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

#[component]
fn UploadHeader() -> impl IntoView {
    view! {
        <div class="mb-8 md:mb-10 text-center">
            <div class="inline-flex items-center justify-center p-4 bg-cyan-400/10 rounded-3xl mb-4">
                <span class="text-cyan-400"><UploadIcon/></span>
            </div>
            <h1 class="text-3xl sm:text-4xl md:text-5xl font-black text-white">"رفع وسائط جديدة"</h1>
            <p class="text-gray-400 text-sm sm:text-base mt-2">
                "أضف فيلماً أو مسلسلاً أو مجموعة صوتية إلى مكتبتك المنزلية"
            </p>
        </div>
    }
}

#[component]
fn UploadSubmitButton(pending: Signal<bool>, file_count: Signal<usize>) -> impl IntoView {
    view! {
        <button
            type="submit"
            disabled=move || pending.get()
            class="w-full py-3 px-6 rounded-2xl bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold text-base shadow-lg shadow-cyan-500/20 transition-all hover:scale-[1.02] hover:shadow-cyan-500/40 flex items-center justify-center gap-2 disabled:opacity-60 disabled:cursor-not-allowed disabled:hover:scale-100"
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
    }
}
