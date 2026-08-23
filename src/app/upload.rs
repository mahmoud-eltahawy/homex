use super::model::MediaType;
use crate::app::{
    icons::{
        AudioIcon, DeleteIcon, DownArrow, MovieIcon, SeriesIcon, SortIcon, UpArrow, UploadIcon,
    },
    model::MediaId,
    resource_view::ResourceView,
};
use leptos::{either::Either, prelude::*};
use leptos_router::{lazy_route, LazyRoute};
use serde::{Deserialize, Serialize};
use web_sys::{wasm_bindgen::JsCast, HtmlInputElement, HtmlSelectElement};

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
    pub id: MediaId,
    pub title: String,
}

#[derive(Clone, Debug)]
pub struct UploadItem {
    pub id: u32,
    pub file: web_sys::File,
    pub title: String,
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

#[server]
async fn upload_media(
    title: String,
    media_type: String,
    description: String,
    is_new_series: bool,
    existing_series_id: Option<i64>,
) -> Result<(), ServerFnError> {
    leptos::logging::log!(
        "Upload: title={title}, type={media_type}, new_series={is_new_series}, existing_id={existing_series_id:?} ,description : {description}"
    );
    Ok(())
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
        let upload_action = ServerAction::<UploadMedia>::new();
        view! {
            <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
                <UploadHeader/>
                <div class=CARD_CLASS>
                    <ActionForm action=upload_action prop:class="space-y-6 md:space-y-8">
                        <UploadContent series_res=this.series/>
                    </ActionForm>
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

    let adapter = move |series_list: Vec<SeriesTitle>| SeriesSettingsProps {
        is_new_series,
        existing_series_id,
        series_list,
    };

    view! {
        <MediaKindSelector media_type/>
        <div class="space-y-4">
            <TitleInput media_type/>
            <DescriptionInput/>
        </div>
        <HiddenFormState media_type is_new_series existing_series_id/>
        {move || match media_type.get() {
            MediaType::Series => Either::Left(view! {
                <SeriesSection series_res=series_res adapter=adapter/>
            }),
            MediaType::Movie => Either::Right(Either::Left(view! {
                <MovieFileInput/>
            })),
            MediaType::AudioGroup => Either::Right(Either::Right(view! {
                <AudioGroupSection/>
            })),
        }}
        <UploadSubmitButton/>
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
        <input type="hidden" name="existing_series_id" value=move || existing_series_id.get().map(|id| id.to_string()).unwrap_or_default()/>
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
    let episode_icon: fn() -> AnyView = || view! { <SeriesIcon/> }.into_any();

    view! {
        <ResourceView
            resource=series_res
            view_fn=SeriesSettings
            adapter=adapter
            context="جارٍ تحميل قائمة المسلسلات"
        />
        <MediaFilesSection
            heading="الحلقات"
            hint="يتم ترقيم الحلقات تلقائياً حسب الترتيب. استخدم الأسهم لإعادة الترتيب أو زر ترتيب للفرز الأبجدي."
            input_id="multiEpisodeInput"
            accept="video/*"
            select_label="اختيار الحلقات"
            number_label="رقم الحلقة"
            title_label="عنوان الحلقة"
            file_label="الملف"
            icon=episode_icon
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
                            <option value={series.id.0.to_string()} class="bg-gray-800">{series.title.clone()}</option>
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
            <FileSelector
                input_id="movieFileInput"
                name="movie_file"
                on_change=on_change
                label="اختر ملف"
                file_name=file_name
            />
        </div>
    }
}

#[component]
fn FileSelector(
    input_id: &'static str,
    name: &'static str,
    on_change: impl Fn(web_sys::Event) + 'static,
    label: &'static str,
    file_name: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div class="flex flex-wrap items-center gap-4">
            <input type="file" name=name id=input_id class="hidden" accept="video/*" on:change=on_change/>
            <label for=input_id
                class="inline-flex items-center gap-2 bg-white/10 hover:bg-white/20 backdrop-blur-md text-white font-medium py-2 px-5 rounded-xl cursor-pointer transition text-sm">
                <UploadIcon/> {label}
            </label>
            <span class="text-sm text-gray-400">
                {move || if file_name.get().is_empty() { "لم يتم اختيار ملف".to_string() } else { file_name.get() }}
            </span>
        </div>
    }
}

#[component]
fn AudioGroupSection() -> impl IntoView {
    let audio_icon: fn() -> AnyView = || view! { <MovieIcon/> }.into_any();

    view! {
        <MediaFilesSection
            heading="المقاطع الصوتية"
            hint="يتم ترقيم المقاطع الصوتية تلقائياً حسب الترتيب. استخدم الأسهم لإعادة الترتيب أو زر ترتيب للفرز الأبجدي."
            input_id="multiAudioInput"
            accept="audio/*"
            select_label="اختيار ملفات صوتية"
            number_label="رقم المقطع"
            title_label="عنوان المقطع الصوتي"
            file_label="الملف"
            icon=audio_icon
        />
    }
}

#[component]
fn MediaFilesSection(
    heading: &'static str,
    hint: &'static str,
    input_id: &'static str,
    accept: &'static str,
    select_label: &'static str,
    number_label: &'static str,
    title_label: &'static str,
    file_label: &'static str,
    icon: fn() -> AnyView,
) -> impl IntoView {
    let items = RwSignal::new(Vec::<UploadItem>::new());
    let next_id = RwSignal::new(1u32);

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
    icon: fn() -> AnyView,
) -> impl IntoView {
    view! {
        <div class="flex flex-wrap items-center justify-between gap-3">
            <h2 class="text-lg font-bold text-white flex items-center gap-2">
                {icon()} {heading}
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
                items.update(|list| list.extend(new_items));
                next_id.update(|id| *id += files.length());
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
                each={move || items.get().into_iter().enumerate().collect::<Vec<_>>()}
                key=|(_, item)| item.id
                let:item
            >
                {move || {
                    let (index, item) = item.clone();
                    view! {
                        <MediaItemRow
                            items=items
                            item_id=item.id
                            index=index
                            number_label=number_label
                            title_label=title_label
                            file_label=file_label
                        />
                    }
                }}
            </For>
        </div>
    }
}

#[component]
fn MediaItemRow(
    items: RwSignal<Vec<UploadItem>>,
    item_id: u32,
    index: usize,
    number_label: &'static str,
    title_label: &'static str,
    file_label: &'static str,
) -> impl IntoView {
    let total = move || items.get().len();
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
            items.update(|list| {
                if let Some(item) = list.iter_mut().find(|e| e.id == item_id) {
                    item.title = input.value();
                }
            });
        }
    };
    let item = move || items.get().into_iter().find(|e| e.id == item_id).unwrap();

    view! {
        <div class=ITEM_CARD_CLASS>
            <div class="flex-1 grid grid-cols-1 sm:grid-cols-3 gap-3 w-full">
                <div>
                    <span class="text-gray-400 text-sm font-medium">{number_label}</span>
                    <div class="text-white font-semibold mt-0.5">{index + 1}</div>
                </div>
                <div class="sm:col-span-2">
                    <label class="text-xs text-gray-400 mb-0.5 block">{title_label}</label>
                    <input type="text"
                        prop:value=move || item().title
                        on:input=title_update
                        placeholder=title_label
                        class="w-full bg-white/10 text-white rounded-lg py-1.5 px-3 text-sm focus:outline-none focus:ring-1 focus:ring-cyan-400"
                    />
                </div>
                <div class="hidden sm:block">
                    <span class="text-xs text-gray-400">{file_label}</span>
                    <div class="text-xs text-gray-300 truncate mt-0.5 max-w-32">
                        {move || item().file.name()}
                    </div>
                </div>
            </div>
            <MediaItemControls
                on_move_up=move_up
                on_move_down=move_down
                on_remove=remove
                index=index
                total=total
            />
        </div>
    }
}

#[component]
fn MediaItemControls(
    on_move_up: impl Fn(web_sys::MouseEvent) + 'static,
    on_move_down: impl Fn(web_sys::MouseEvent) + 'static,
    on_remove: impl Fn(web_sys::MouseEvent) + 'static,
    index: usize,
    total: impl Fn() -> usize + Send + 'static,
) -> impl IntoView {
    view! {
        <div class="flex items-center gap-1 mt-1 sm:mt-0">
            <button on:click=on_move_up disabled=move || index == 0
                class=ICON_BTN_CLASS title="نقل للأعلى">
                <UpArrow/>
            </button>
            <button on:click=on_move_down disabled=move || index + 1 == total()
                class=ICON_BTN_CLASS title="نقل للأسفل">
                <DownArrow/>
            </button>
            <button on:click=on_remove
                class="text-red-400 hover:text-red-300 transition p-1" title="حذف">
                <DeleteIcon/>
            </button>
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
            <p class="text-gray-400 text-sm sm:text-base mt-2">"أضف فيلمًا أو مسلسلًا أو مجموعة صوتية إلى مكتبتك المنزلية"</p>
        </div>
    }
}

#[component]
fn UploadSubmitButton() -> impl IntoView {
    view! {
        <button type="submit"
            class="w-full py-3 px-6 rounded-2xl bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold text-base shadow-lg shadow-cyan-500/20 transition-all hover:scale-[1.02] hover:shadow-cyan-500/40 flex items-center justify-center gap-2">
            <UploadIcon/> "رفع الوسائط"
        </button>
    }
}
