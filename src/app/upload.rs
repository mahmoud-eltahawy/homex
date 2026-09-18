use super::model::MediaType;
use crate::app::{
    icons::{
        AudioIcon, DeleteIcon, DownArrow, MovieIcon, SeriesIcon, SortIcon, UpArrow, UploadIcon,
    },
    resource_view::ResourceView,
};
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
pub struct SeriesTitle {
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

#[server]
async fn fetch_series_titles() -> Result<Vec<SeriesTitle>, ServerFnError> {
    use crate::app::model::Series;
    use crate::app::{delay, mockary::mock_series};
    delay(200).await;
    let list = mock_series();
    Ok(list
        .into_iter()
        .map(|Series { id, title, .. }| SeriesTitle { id, title })
        .collect())
}

#[server(
    input = MultipartFormData,    
)]
pub async fn upload_media(data: MultipartData) -> Result<UploadResult, ServerFnError> {
    // MultipartData wraps axum::extract::Multipart on the server.
    let mut multipart = data.into_inner().unwrap();

    let mut title = String::new();
    let mut media_type = String::new();
    let mut description = String::new();
    let mut is_new_series = true;
    let mut existing_series_id: Option<i64> = None;
    let mut movie_file: Option<(String, usize)> = None;
    // (index, filename, size)
    let mut files: Vec<(usize, String, usize)> = Vec::new();
    let mut titles: std::collections::HashMap<usize, String> = std::collections::HashMap::new();

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().map(String::from).unwrap_or_default();

        if name == "title" {
            title = field.text().await?;
        } else if name == "media_type" {
            media_type = field.text().await?;
        } else if name == "description" {
            description = field.text().await?;
        } else if name == "is_new_series" {
            is_new_series = field.text().await? == "true";
        } else if name == "existing_series_id" {
            let text = field.text().await?;
            existing_series_id = text.parse().ok();
        } else if name == "movie_file" {
            let file_name = field.file_name().map(String::from).unwrap_or_default();
            let bytes = field.bytes().await?;
            movie_file = Some((file_name, bytes.len()));
        } else if let Some(idx_str) = name.strip_prefix("file_title_") {
            // NOTE: must be checked before `file_` prefix
            if let Ok(idx) = idx_str.parse::<usize>() {
                titles.insert(idx, field.text().await?);
            } else {
                let _ = field.bytes().await?;
            }
        } else if let Some(idx_str) = name.strip_prefix("file_") {
            if let Ok(idx) = idx_str.parse::<usize>() {
                let file_name = field.file_name().map(String::from).unwrap_or_default();
                let bytes = field.bytes().await?;
                files.push((idx, file_name, bytes.len()));
            } else {
                let _ = field.bytes().await?;
            }
        } else {
            let _ = field.bytes().await?;
        }
    }

    // TODO: replace with real persistence. For now, log what we got.
    leptos::logging::log!(
        "[upload] title={title:?} type={media_type:?} desc_len={} new_series={is_new_series} existing_id={existing_series_id:?}",
        description.len()
    );
    if let Some((name, size)) = &movie_file {
        leptos::logging::log!("[upload]   movie_file: {name} ({size} bytes)");
    }
    files.sort_by_key(|(idx, _, _)| *idx);
    for (idx, name, size) in &files {
        let t = titles.get(idx).map(String::as_str).unwrap_or("(no title)");
        leptos::logging::log!("[upload]   file[{idx}]: {name} ({size} bytes) title={t:?}");
    }

    let total = files.len() + usize::from(movie_file.is_some());
    Ok(UploadResult {
        success: true,
        message: format!("تم استلام {total} ملف بنجاح"),
    })
}

pub struct UploadPage {
    series: Resource<Result<Vec<SeriesTitle>, ServerFnError>>,
}

#[lazy_route]
impl LazyRoute for UploadPage {
    fn data() -> Self {
        let series = Resource::new(|| (), |_| async move { fetch_series_titles().await });
        Self { series }
    }

    fn view(this: Self) -> AnyView {
        view! {
            <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
                <UploadHeader/>
                <div class=CARD_CLASS>
                    <UploadContent series_res=this.series/>
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

#[component]
fn UploadContent(series_res: Resource<Result<Vec<SeriesTitle>, ServerFnError>>) -> impl IntoView {
    let media_type = RwSignal::new(MediaType::Series);
    let is_new_series = RwSignal::new(true);
    let existing_series_id = RwSignal::new(None::<i64>);

    // Shared file state for the currently active media section.
    let items = RwSignal::new(Vec::<UploadItem>::new());
    let next_id = RwSignal::new(1u32);

    // Clear files when the user switches media kind.
    Effect::new(move |prev: Option<MediaType>| {
        let current = media_type.get();
        if prev.is_some() && prev != Some(current) {
            items.set(Vec::new());
            next_id.set(1);
        }
        current
    });

    let upload_action = Action::new_local(|data : &FormData| {
        upload_media(data.clone().into())
    });

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();

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

        upload_action.dispatch(form_data);
    };

    let adapter = move |series_list: Vec<SeriesTitle>| SeriesSettingsProps {
        is_new_series,
        existing_series_id,
        series_list,
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
        <form on:submit=on_submit class="space-y-6 md:space-y-8">
            <MediaKindSelector media_type/>
            <div class="space-y-4">
                <TitleInput media_type/>
                <DescriptionInput/>
            </div>
            <HiddenFormState media_type is_new_series existing_series_id/>

            {move || match media_type.get() {
                MediaType::Series => view! {
                    <>
                        <SeriesSection
                            series_res=series_res
                            adapter=adapter
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
                }.into_any(),
                MediaType::Movie => view! {
                    <MovieFileInput/>
                }.into_any(),
                MediaType::AudioGroup => view! {
                    <MediaFilesSection
                        items=items
                        next_id=next_id
                        heading="المقاطع الصوتية"
                        hint="يتم ترقيم المقاطع تلقائياً حسب الترتيب. استخدم الأسهم لإعادة الترتيب أو زر ترتيب للفرز الأبجدي."
                        input_id="multiAudioInput"
                        accept="audio/*"
                        select_label="اختيار ملفات صوتية"
                        number_label="رقم المقطع"
                        title_label="عنوان المقطع الصوتي"
                        file_label="الملف"
                        icon=AudioIcon()
                    />
                }.into_any(),
            }}

            {result_view}

            <UploadSubmitButton pending=upload_action.pending().into()/>
        </form>
    }
}

#[component]
fn HiddenFormState(
    media_type: RwSignal<MediaType>,
    is_new_series: RwSignal<bool>,
    existing_series_id: RwSignal<Option<i64>>,
) -> impl IntoView {
    view! {
        <input type="hidden" name="media_type" value=move || media_type.get().to_string()/>
        <input type="hidden" name="is_new_series" value=move || is_new_series.get().to_string()/>
        <input
            type="hidden"
            name="existing_series_id"
            value=move || existing_series_id.get().map(|id| id.to_string()).unwrap_or_default()
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
    view! {
        <div>
            <label class="block text-sm font-medium text-gray-300 mb-1.5">"العنوان *"</label>
            <input type="text" name="title" required placeholder=placeholder class=INPUT_CLASS/>
        </div>
    }
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
fn SeriesSection(
    series_res: Resource<Result<Vec<SeriesTitle>, ServerFnError>>,
    adapter: impl Fn(Vec<SeriesTitle>) -> SeriesSettingsProps + Send + 'static,
) -> impl IntoView {
    view! {
        <ResourceView
            resource=series_res
            view_fn=SeriesSettings
            adapter=adapter
        />
    }
}

#[component]
fn SeriesSettings(
    is_new_series: RwSignal<bool>,
    existing_series_id: RwSignal<Option<i64>>,
    series_list: Vec<SeriesTitle>,
) -> impl IntoView {
    view! {
        <div class="space-y-4">
            <SeriesTypeToggle is_new_series=is_new_series existing_series_id=existing_series_id/>
            <ExistingSeriesSelect
                is_new_series=is_new_series
                existing_series_id=existing_series_id
                series_list=series_list
            />
        </div>
    }
}

#[component]
fn SeriesTypeToggle(
    is_new_series: RwSignal<bool>,
    existing_series_id: RwSignal<Option<i64>>,
) -> impl IntoView {
    view! {
        <div class="flex flex-wrap items-center gap-4">
            <label class="text-sm font-medium text-gray-300">نوع المسلسل:</label>
            <div class="inline-flex bg-white/5 rounded-xl p-0.5">
                <button type="button"
                    on:click=move |_| { is_new_series.set(true); existing_series_id.set(None); }
                    class=move || toggle_btn_class(is_new_series.get())>
                    جديد
                </button>
                <button type="button"
                    on:click=move |_| is_new_series.set(false)
                    class=move || toggle_btn_class(!is_new_series.get())>
                    موجود
                </button>
            </div>
        </div>
    }
}

#[component]
fn ExistingSeriesSelect(
    is_new_series: RwSignal<bool>,
    existing_series_id: RwSignal<Option<i64>>,
    series_list: Vec<SeriesTitle>,
) -> impl IntoView {
    move || {
        if !is_new_series.get() {
            Some(view! {
                <div>
                    <label class="block text-sm font-medium text-gray-300 mb-1.5">اختر المسلسل الموجود</label>
                    <select
                        name="existing_series_id_select"
                        on:change=move |ev| {
                            if let Some(sel) = ev.target().and_then(|t| t.dyn_into::<HtmlSelectElement>().ok()) {
                                existing_series_id.set(sel.value().parse().ok());
                            }
                        }
                        class=INPUT_CLASS
                    >
                        <option value="" class="bg-gray-800">"-- اختر --"</option>
                        {series_list.iter().map(|series| view! {
                            <option value={series.id.to_string()} class="bg-gray-800">{series.title.clone()}</option>
                        }).collect_view()}
                    </select>
                </div>
            })
        } else {
            None
        }
    }
}

#[component]
fn MovieFileInput() -> impl IntoView {
    let file_name = RwSignal::new(String::new());
    let on_change = move |ev: web_sys::Event| {
        if let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        {
            if let Some(file) = input.files().and_then(|f| f.get(0)) {
                file_name.set(file.name());
            }
        }
    };

    view! {
        <div>
            <label class="block text-sm font-medium text-gray-300 mb-1.5">"ملف الفيلم"</label>
            <div class="flex flex-wrap items-center gap-4">
                <input
                    type="file"
                    name="movie_file"
                    id="movieFileInput"
                    class="hidden"
                    accept="video/*"
                    on:change=on_change
                />
                <label for="movieFileInput"
                    class="inline-flex items-center gap-2 bg-white/10 hover:bg-white/20 backdrop-blur-md text-white font-medium py-2 px-5 rounded-xl cursor-pointer transition text-sm">
                    <UploadIcon/> "اختر ملف"
                </label>
                <span class="text-sm text-gray-400">
                    {move || if file_name.get().is_empty() {
                        "لم يتم اختيار ملف".to_string()
                    } else {
                        file_name.get()
                    }}
                </span>
            </div>
        </div>
    }
}

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

// Index is now computed reactively from `items`, so it stays correct
// after reordering. The row itself does NOT remount when the list changes
// order (keyed by id), so focus is preserved while typing titles.
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
fn UploadSubmitButton(pending: Signal<bool>) -> impl IntoView {
    view! {
        <button
            type="submit"
            disabled=move || pending.get()
            class="w-full py-3 px-6 rounded-2xl bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold text-base shadow-lg shadow-cyan-500/20 transition-all hover:scale-[1.02] hover:shadow-cyan-500/40 flex items-center justify-center gap-2 disabled:opacity-60 disabled:cursor-not-allowed disabled:hover:scale-100"
        >
            <UploadIcon/>
            {move || if pending.get() { "جاري الرفع..." } else { "رفع الوسائط" }}
        </button>
    }
}
