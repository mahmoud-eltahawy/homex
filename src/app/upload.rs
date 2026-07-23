use super::{EpUpload, MediaType};
use crate::app::{
    DeleteIcon, DownArrow, MediaId, MovieIcon, SeriesIcon, SortIcon, UpArrow, UploadIcon,
};
use leptos::{either::Either, prelude::*};
use leptos_router::{lazy_route, LazyRoute};
use serde::{Deserialize, Serialize};
use web_sys::{wasm_bindgen::JsCast, HtmlInputElement, HtmlSelectElement};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeriesTitle {
    pub id: MediaId,
    pub title: String,
}

#[server]
async fn fetch_series_titles() -> Result<Vec<SeriesTitle>, ServerFnError> {
    use crate::app::{delay, series::listing::mock_series, Series};
    delay(200).await;
    let list = mock_series();
    let res = list
        .into_iter()
        .map(|Series { id, title, .. }| SeriesTitle { id, title })
        .collect();
    Ok(res)
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
        Upload(UploadProps {
            series_res: this.series,
        })
        .into_any()
    }
}

#[component]
fn Upload(series_res: Resource<Result<Vec<SeriesTitle>, ServerFnError>>) -> impl IntoView {
    let title = RwSignal::new(String::new());
    let media_type = RwSignal::new(MediaType::Series);
    let description = RwSignal::new(String::new());
    let movie_file = RwSignal::new(None::<web_sys::File>);
    let is_new_series = RwSignal::new(true);
    let existing_series_id = RwSignal::new(None::<i64>);
    let series_list = Memo::new(move |_| series_res.get().and_then(|x| x.ok()).unwrap_or_default());
    let episodes = RwSignal::new(Vec::<EpUpload>::new());
    let next_id = RwSignal::new(1u32);
    let disabled = Signal::derive(move || {
        !is_new_series.get() && matches!(media_type.get(), MediaType::Series)
    });
    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        if matches!(media_type.get(), MediaType::Series) && episodes.get().is_empty() {
            return;
        }
        title.set(String::new());
        description.set(String::new());
        movie_file.set(None);
        episodes.set(Vec::new());
        next_id.set(1);
        is_new_series.set(true);
        existing_series_id.set(None);
    };
    let media_input = move || match media_type.get() {
        MediaType::Series => Either::Left(view! {
            <Suspense fallback=|| view! { <div class="text-gray-400 text-sm">"جارٍ تحميل قائمة المسلسلات..."</div> }>
                <SeriesSettings is_new_series=is_new_series existing_series_id=existing_series_id series_list=Signal::derive(move || series_list.get())/>
            </Suspense>
            <EpisodesSection episodes=episodes next_id=next_id/>
        }),
        MediaType::Movie => Either::Right(view! { <MovieFileInput movie_file=movie_file/> }),
    };
    view! {
        <div
            class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8"
        >
            <UploadHeader/>
            <div
                class="backdrop-blur-xl bg-white/5 rounded-3xl border border-white/10 p-6 md:p-8 shadow-2xl"
            >
                <form
                    on:submit=handle_submit
                    class="space-y-6 md:space-y-8"
                >
                    <MediaKindSelector
                         media_type=media_type
                     />
                    <div
                        class="space-y-4"
                    >
                        <TitleInput
                            title=title
                            disabled=disabled
                        />
                        <DescriptionInput description=description/>
                    </div>
                    {media_input}
                    <UploadSubmitButton/>
                </form>
            </div>
        </div>
    }
}

#[component]
fn MediaKindSelector(media_type: RwSignal<MediaType>) -> impl IntoView {
    let class1 = move || {
        format!("px-4 sm:px-6 py-2 rounded-xl text-sm font-medium transition flex items-center gap-2 {}",
        if matches!(media_type.get(), MediaType::Series) { "bg-purple-500/20 text-purple-400 shadow-lg shadow-purple-500/10" } else { "text-gray-400 hover:text-white" })
    };
    let class2 = move || {
        format!("px-4 sm:px-6 py-2 rounded-xl text-sm font-medium transition flex items-center gap-2 {}",
        if matches!(media_type.get(), MediaType::Movie) { "bg-cyan-500/20 text-cyan-400 shadow-lg shadow-cyan-500/10" } else { "text-gray-400 hover:text-white" })
    };
    view! {
    <div
        class="flex justify-center"
    >
        <div
            class="inline-flex bg-white/5 rounded-2xl p-1" role="group"
        >
            <button
                type="button"
                on:click=move |_| media_type.set(MediaType::Series)
                class=class1
            >
                <SeriesIcon/>
                "مسلسل"
            </button>
            <button
                type="button"
                on:click=move |_| media_type.set(MediaType::Movie)
                class=class2
            >
                <MovieIcon/>
                "فيلم"
            </button>
        </div>
    </div>
    }
}

#[component]
fn TitleInput(title: RwSignal<String>, disabled: Signal<bool>) -> impl IntoView {
    let class = move || {
        format!("w-full bg-white/10 backdrop-blur-md text-white placeholder-gray-500 rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/20 transition {}",
        if disabled.get() { "opacity-60 cursor-not-allowed" } else { "" })
    };
    view! {
        <div>
            <label
                class="block text-sm font-medium text-gray-300 mb-1.5"
            >
                "العنوان *"
            </label>
            <input
                type="text"
                prop:value=title
                on:input=move |ev| title.set(event_target_value(&ev))
                required
                placeholder="مثال: Breaking Bad"
                class=class
                disabled=disabled.get()
            />
        </div>
    }
}

#[component]
fn DescriptionInput(description: RwSignal<String>) -> impl IntoView {
    view! {
        <div>
            <label class="block text-sm font-medium text-gray-300 mb-1.5">"الوصف (اختياري)"</label>
            <textarea prop:value=description on:input=move |ev| description.set(event_target_value(&ev))
                rows=3 placeholder="وصف مختصر (اختياري)..."
                class="w-full bg-white/10 backdrop-blur-md text-white placeholder-gray-500 rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/20 transition resize-none"
            />
        </div>
    }
}

#[component]
fn SeriesSettings(
    is_new_series: RwSignal<bool>,
    existing_series_id: RwSignal<Option<i64>>,
    series_list: Signal<Vec<SeriesTitle>>,
) -> impl IntoView {
    view! {
        <div class="space-y-4">
            <div class="flex flex-wrap items-center gap-4">
                <label class="text-sm font-medium text-gray-300">نوع المسلسل:</label>
                <div class="inline-flex bg-white/5 rounded-xl p-0.5">
                    <button type="button"
                        on:click=move |_| { is_new_series.set(true); existing_series_id.set(None); }
                        class=move || format!("px-3 py-1.5 rounded-lg text-sm font-medium transition {}",
                            if is_new_series.get() { "bg-cyan-500/20 text-cyan-400" } else { "text-gray-400 hover:text-white" })>
                        جديد
                    </button>
                    <button type="button"
                        on:click=move |_| is_new_series.set(false)
                        class=move || format!("px-3 py-1.5 rounded-lg text-sm font-medium transition {}",
                            if !is_new_series.get() { "bg-cyan-500/20 text-cyan-400" } else { "text-gray-400 hover:text-white" })>
                        موجود
                    </button>
                </div>
            </div>
            {move || if !is_new_series.get() {
                Some(view! {
                    <div>
                        <label class="block text-sm font-medium text-gray-300 mb-1.5">اختر المسلسل الموجود</label>
                        <select
                            prop:value=move || existing_series_id.get().map(|id| id.to_string()).unwrap_or_default()
                            on:change=move |ev| {
                                if let Some(sel) = ev.target().and_then(|t| t.dyn_into::<HtmlSelectElement>().ok()) {
                                    existing_series_id.set(sel.value().parse().ok());
                                }
                            }
                            class="w-full bg-white/10 backdrop-blur-md text-white rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50"
                        >
                            <option value="" class="bg-gray-800">"-- اختر --"</option>
                            <For each={move || series_list.get()} key=|m| m.id let:series>
                                <option value={series.id.0.to_string()} class="bg-gray-800">{series.title}</option>
                            </For>
                        </select>
                    </div>
                })
            } else {
                None
            }}
        </div>
    }
}

#[component]
fn MovieFileInput(movie_file: RwSignal<Option<web_sys::File>>) -> impl IntoView {
    let on_change = move |ev: web_sys::Event| {
        if let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        {
            movie_file.set(input.files().and_then(|f| f.get(0)));
        }
    };
    let file_name = move || {
        movie_file
            .get()
            .as_ref()
            .map(|f| f.name())
            .unwrap_or_else(|| "لم يتم اختيار ملف".to_string())
    };
    view! {
        <div>
            <label
                class="block text-sm font-medium text-gray-300 mb-1.5"
            >
                "ملف الفيل"
            </label>
            <div
                class="flex flex-wrap items-center gap-4"
            >
                <input
                    type="file"
                    id="movieFileInput"
                    class="hidden"
                    accept="video/*"
                    on:change=on_change
                />
                <label
                    for="movieFileInput"
                    class="inline-flex items-center gap-2 bg-white/10 hover:bg-white/20 backdrop-blur-md text-white font-medium py-2 px-5 rounded-xl cursor-pointer transition text-sm"
                >
                    <UploadIcon/>
                    "اختر ملف"
                </label>
                <span
                    class="text-sm text-gray-400"
                >
                    {file_name}
                </span>
            </div>
        </div>
    }
}

#[component]
fn EpisodeItem(episodes: RwSignal<Vec<EpUpload>>, ep_id: u32, index: usize) -> impl IntoView {
    let total = move || episodes.get().len();
    let remove = move |_| episodes.update(|eps| eps.retain(|e| e.id != ep_id));
    let move_up = move |_| {
        episodes.update(|eps| {
            if let Some(pos) = eps.iter().position(|e| e.id == ep_id) {
                if pos > 0 {
                    eps.swap(pos, pos - 1);
                }
            }
        })
    };
    let move_down = move |_| {
        episodes.update(|eps| {
            if let Some(pos) = eps.iter().position(|e| e.id == ep_id) {
                if pos + 1 < eps.len() {
                    eps.swap(pos, pos + 1);
                }
            }
        })
    };
    let title_update = move |ev: web_sys::Event| {
        if let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        {
            episodes.update(|eps| {
                if let Some(ep) = eps.iter_mut().find(|e| e.id == ep_id) {
                    ep.title = input.value();
                }
            });
        }
    };
    let ep = move || episodes.get().into_iter().find(|e| e.id == ep_id).unwrap();
    view! {
    <div
        class="bg-white/5 backdrop-blur-sm rounded-xl border border-white/10 p-4 flex flex-col sm:flex-row gap-3 items-start"
    >
        <div
            class="flex-1 grid grid-cols-1 sm:grid-cols-3 gap-3 w-full"
        >
            <div>
                <span
                    class="text-gray-400 text-sm font-medium">رقم الحلقة</span><div class="text-white font-semibold mt-0.5"
                >
                {index + 1}
            </div>
        </div>
        <div
            class="sm:col-span-2"
        >
            <label
                class="text-xs text-gray-400 mb-0.5 block"
            >
                "عنوان الحلقة"
            </label>
            <input
                type="text"
                prop:value=move || ep().title
                on:input=title_update
                placeholder="عنوان الحلقة"
                class="w-full bg-white/10 text-white rounded-lg py-1.5 px-3 text-sm focus:outline-none focus:ring-1 focus:ring-cyan-400"
            />
        </div>
        <div
            class="hidden sm:block"
        >
            <span
                class="text-xs text-gray-400"
            >
                "الملف"
            </span>
            <div
                class="text-xs text-gray-300 truncate mt-0.5 max-w-32"
            >
                {move || ep().file.name()}
            </div>
        </div>
        </div>
        <div
            class="flex items-center gap-1 mt-1 sm:mt-0"
        >
            <button
                on:click=move_up
                disabled=move || index == 0
                class="text-gray-400 hover:text-white transition disabled:opacity-30 p-1"
                title="نقل للأعلى"
            >
                <UpArrow/>
            </button>
            <button
                on:click=move_down
                disabled=move || index + 1 == total()
                class="text-gray-400 hover:text-white transition disabled:opacity-30 p-1"
                title="نقل للأسفل"
            >
                <DownArrow/>
            </button>
            <button
                on:click=remove
                class="text-red-400 hover:text-red-300 transition p-1"
                title="حذف الحلقة"
            >
                <DeleteIcon/>
            </button>
        </div>
    </div>
    }
}

#[component]
fn EpisodesToolbar(episodes: RwSignal<Vec<EpUpload>>, next_id: RwSignal<u32>) -> impl IntoView {
    let sort = move |_| episodes.update(|eps| eps.sort_by_key(|x| x.file.name()));
    let file_handler = move |ev: web_sys::Event| {
        if let Some(input) = ev
            .target()
            .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        {
            if let Some(files) = input.files() {
                let mut new_eps: Vec<EpUpload> = (0..files.length())
                    .filter_map(|i| files.get(i))
                    .map(|file| {
                        let name = file.name();
                        let title = name.rsplitn(2, '.').last().unwrap_or(&name).to_string();
                        EpUpload {
                            id: next_id.get(),
                            file,
                            title,
                        }
                    })
                    .collect();
                new_eps.sort_by_key(|x| x.file.name());
                episodes.update(|eps| eps.extend(new_eps));
                next_id.update(|id| *id += files.length());
                input.set_value("");
            }
        }
    };
    view! {
        <div
            class="flex flex-wrap items-center justify-between gap-3"
        >
            <h2
                class="text-lg font-bold text-white flex items-center gap-2"
            >
                <SeriesIcon/>
                 "الحلقات"
             </h2>
            <div
                class="flex flex-wrap items-center gap-2"
            >
                <input
                    type="file"
                    id="multiEpisodeInput"
                    class="hidden"
                    multiple
                    accept="video/*"
                    on:change=file_handler
                />
                <label
                    for="multiEpisodeInput"
                    class="inline-flex items-center gap-1.5 bg-green-500/20 hover:bg-green-500/30 backdrop-blur-md text-green-300 font-medium py-1.5 px-3 rounded-lg cursor-pointer transition text-sm"
                >
                    <UploadIcon/>
                    "اختيار ملفات"
                </label>
                <button
                    type="button"
                    on:click=sort
                    class="inline-flex items-center gap-1.5 bg-white/10 hover:bg-white/20 backdrop-blur-md text-white font-medium py-1.5 px-3 rounded-lg transition text-sm"
                >
                    <SortIcon/>
                    "ترتيب"
                </button>
            </div>
        </div>
    }
}

#[component]
fn EpisodesSection(episodes: RwSignal<Vec<EpUpload>>, next_id: RwSignal<u32>) -> impl IntoView {
    view! {
        <div class="space-y-4">
            <EpisodesToolbar episodes=episodes next_id=next_id/>
            <div class="space-y-3 max-h-80 overflow-y-auto p-1">
                <For
                    each={move || episodes.get().into_iter().enumerate().collect::<Vec<_>>()}
                    key=|(_, ep)| ep.id
                    let:item
                >
                    {move || {
                        let (i, ep) = item.clone();
                        view! { <EpisodeItem episodes=episodes ep_id=ep.id index=i/> }
                    }}
                </For>
            </div>
            <p class="text-xs text-gray-500">
                "يتم ترقيم الحلقات تلقائياً حسب الترتيب. استخدم الأسهم لإعادة الترتيب أو زر ترتيب للفرز الأبجدي."
            </p>
        </div>
    }
}

#[component]
fn UploadHeader() -> impl IntoView {
    view! {
        <div
            class="mb-8 md:mb-10 text-center"
        >
            <div
                class="inline-flex items-center justify-center p-4 bg-cyan-400/10 rounded-3xl mb-4"
            >
                <span
                    class="text-cyan-400"
                >
                    <UploadIcon/>
                </span>
            </div>
            <h1
                class="text-3xl sm:text-4xl md:text-5xl font-black text-white"
            >
                "رفع وسائط جديدة"
            </h1>
            <p
                class=
                    "text-gray-400 text-sm sm:text-base mt-2"
            >
                    "أضف فيلمًا أو مسلسلًا إلى مكتبتك المنزلية"
            </p>
        </div>
    }
}

#[component]
fn UploadSubmitButton() -> impl IntoView {
    view! {
        <button
            type="submit"
            class="w-full py-3 px-6 rounded-2xl bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold text-base shadow-lg shadow-cyan-500/20 transition-all hover:scale-[1.02] hover:shadow-cyan-500/40 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:scale-100 flex items-center justify-center gap-2"
        >
            <UploadIcon/>
            "رفع الوسائط"
        </button>
    }
}
