#[cfg(feature = "ssr")]
use crate::app::server::{poster::write_poster, upload::extension_of};
use crate::app::{
    icons::{DeleteIcon, EditIcon, UploadIcon},
    resource_view::ResourceView,
};
use leptos::either::Either;
use leptos::html;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use leptos_router::{LazyRoute, lazy_route};
use serde::{Deserialize, Serialize};
use server_fn::codec::{MultipartData, MultipartFormData};
use web_sys::{FormData, HtmlInputElement, Url, wasm_bindgen::JsCast};

const INPUT_CLASS: &str = "w-full bg-white/10 backdrop-blur-md text-white placeholder-gray-500 rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/20 transition";
const TEXTAREA_CLASS: &str = "w-full bg-white/10 backdrop-blur-md text-white placeholder-gray-500 rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/20 transition resize-none";
const CARD_CLASS: &str =
    "backdrop-blur-xl bg-white/5 rounded-3xl border border-white/10 p-6 md:p-8 shadow-2xl";
const UPLOAD_BTN_CLASS: &str = "inline-flex items-center gap-1.5 bg-green-500/20 hover:bg-green-500/30 backdrop-blur-md text-green-300 font-medium py-1.5 px-3 rounded-lg cursor-pointer transition text-sm";
const IMAGE_ACCEPT: &str = "image/jpeg,image/png,image/webp,.jpg,.jpeg,.png,.webp";

// ─── Entity kind ──────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetadataKind {
    Movie,
    Series,
    AudioGroup,
}

impl MetadataKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Movie => "movie",
            Self::Series => "series",
            Self::AudioGroup => "audio",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Movie => "فيلم",
            Self::Series => "مسلسل",
            Self::AudioGroup => "مجموعة صوتية",
        }
    }
    pub fn detail_href(self, id: u64) -> String {
        match self {
            Self::Movie => format!("/movie/detail/{id}"),
            Self::Series => format!("/series/detail/{id}"),
            Self::AudioGroup => format!("/audio/detail/{id}"),
        }
    }
    #[cfg(feature = "ssr")]
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "movie" => Some(Self::Movie),
            "series" => Some(Self::Series),
            "audio" => Some(Self::AudioGroup),
            _ => None,
        }
    }
    #[cfg(feature = "ssr")]
    fn table(self) -> &'static str {
        match self {
            Self::Movie => "movies",
            Self::Series => "series",
            Self::AudioGroup => "audio_groups",
        }
    }

    #[cfg(feature = "ssr")]
    fn poster_subdir(self) -> &'static str {
        match self {
            Self::Movie => "movies",
            Self::Series => "series",
            Self::AudioGroup => "audio",
        }
    }
}

// ─── DTOs ─────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MetadataSnapshot {
    pub title: String,
    pub description: Option<String>,
    pub poster: Option<String>,
}

// ─── Server functions ─────────────────────────────────────────────────────

#[server]
pub async fn load_metadata(kind: String, id: u64) -> Result<MetadataSnapshot, ServerFnError> {
    use crate::app::server::AppState;
    use sqlx::AssertSqlSafe;

    let state: AppState = expect_context();

    let kind = MetadataKind::from_str(&kind).ok_or_else(|| ServerFnError::new("نوع غير معروف"))?;

    #[derive(sqlx::FromRow)]
    struct Row {
        title: String,
        description: Option<String>,
        poster: Option<String>,
    }

    let sql = AssertSqlSafe(format!(
        "SELECT title, description, poster FROM {} WHERE id = ?",
        kind.table()
    ));
    let row: Row = sqlx::query_as(sql)
        .bind(id as i64)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
        .ok_or_else(|| ServerFnError::new("العنصر غير موجود"))?;

    Ok(MetadataSnapshot {
        title: row.title,
        description: row.description,
        poster: row.poster,
    })
}

#[server(input = MultipartFormData)]
pub async fn update_metadata(data: MultipartData) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    use sqlx::AssertSqlSafe;

    let state: AppState = expect_context();
    let mut multipart = data.into_inner().unwrap();

    let mut kind_str = String::new();
    let mut id: i64 = 0;
    let mut title = String::new();
    let mut description = String::new();
    let mut remove_poster = false;
    let mut new_poster: Option<(String, Vec<u8>)> = None;

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().map(String::from).unwrap_or_default();
        match name.as_str() {
            "kind" => kind_str = field.text().await?,
            "id" => id = field.text().await?.parse().unwrap_or(0),
            "title" => title = field.text().await?,
            "description" => description = field.text().await?,
            "remove_poster" => remove_poster = field.text().await? == "true",
            "poster_file" => {
                let fname = field.file_name().map(String::from).unwrap_or_default();
                let bytes = field.bytes().await?.to_vec();
                if !bytes.is_empty() {
                    new_poster = Some((extension_of(&fname), bytes));
                }
            }
            _ => {
                let _ = field.bytes().await?;
            }
        }
    }

    if id <= 0 {
        return Err(ServerFnError::new("id غير صالح"));
    }
    if title.trim().is_empty() {
        return Err(ServerFnError::new("العنوان مطلوب"));
    }

    let kind =
        MetadataKind::from_str(&kind_str).ok_or_else(|| ServerFnError::new("نوع غير معروف"))?;
    let table = kind.table();

    let poster_update: Option<Option<String>> = if let Some((ext, bytes)) = new_poster {
        let url = write_poster(
            &state.config.storage.data_dir,
            kind.poster_subdir(),
            id,
            &ext,
            &bytes,
        )
        .await?;
        Some(Some(url))
    } else if remove_poster {
        Some(None)
    } else {
        None
    };

    let description_opt = if description.trim().is_empty() {
        None
    } else {
        Some(description.as_str())
    };

    // Title and description are always touched.
    sqlx::query(AssertSqlSafe(format!(
        "UPDATE {table} SET title = ?, description = ? WHERE id = ?"
    )))
    .bind(&title)
    .bind(description_opt)
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Poster is a separate concern, and is only touched when the user
    // actually changed it. Two statements beat one interleaved match —
    // SQLite is in-process, the extra round-trip is negligible.
    match poster_update {
        Some(Some(url)) => {
            sqlx::query(AssertSqlSafe(format!(
                "UPDATE {table} SET poster = ? WHERE id = ?"
            )))
            .bind(&url)
            .bind(id)
            .execute(&state.db)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        }
        Some(None) => {
            sqlx::query(AssertSqlSafe(format!(
                "UPDATE {table} SET poster = NULL WHERE id = ?"
            )))
            .bind(id)
            .execute(&state.db)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
        }
        None => {}
    }

    Ok(())
}

// ─── Shared editor component ──────────────────────────────────────────────

#[component]
pub fn MetadataEditor(kind: MetadataKind, id: u64) -> impl IntoView {
    let snapshot = Resource::new(
        move || (kind, id),
        |(k, i)| load_metadata(k.as_str().to_string(), i),
    );

    let adapter = move |snap: MetadataSnapshot| EditorFormProps {
        kind,
        id,
        snapshot: snap,
    };

    view! {
        <div class="max-w-3xl mx-auto px-4 sm:px-6 lg:px-8">
            <EditorHeader kind id/>
            <div class=CARD_CLASS>
                <ResourceView
                    resource=snapshot
                    view_fn=EditorForm
                    adapter=adapter
                />
            </div>
        </div>
    }
}

#[component]
fn EditorHeader(kind: MetadataKind, id: u64) -> impl IntoView {
    let back_href = kind.detail_href(id);
    let label = kind.label();
    view! {
        <div class="mb-6 md:mb-8">
            <a
                href=back_href
                class="inline-flex items-center gap-1 text-sm text-gray-400 hover:text-white transition mb-3"
            >
                <span class="text-cyan-400">"←"</span>
                <span>"رجوع"</span>
            </a>
            <h1 class="text-2xl sm:text-3xl md:text-4xl font-black text-white flex items-center gap-3">
                <span class="text-cyan-400"><EditIcon/></span>
                <span>"تعديل بيانات "{label}</span>
            </h1>
            <p class="text-gray-400 text-sm mt-2">
                "يمكنك تعديل العنوان والوصف والصورة دون الحاجة إلى رفع الملفات مرة أخرى."
            </p>
        </div>
    }
}

#[component]
fn EditorForm(kind: MetadataKind, id: u64, snapshot: MetadataSnapshot) -> impl IntoView {
    // Seeded once from the snapshot. Signals own the form after that.
    let title = RwSignal::new(snapshot.title);
    let description = RwSignal::new(snapshot.description.clone().unwrap_or_default());
    let initial_poster = snapshot.poster.clone();

    let poster_file = RwSignal::new(None::<web_sys::File>);
    let preview_url = RwSignal::new(None::<String>);
    let remove_poster = RwSignal::new(false);

    let update_action = Action::new_local(|data: &FormData| update_metadata(data.clone().into()));

    let form_ref = NodeRef::<html::Form>::new();

    let on_submit = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        if update_action.pending().get_untracked() {
            return;
        }

        let Some(form) = form_ref.get() else { return };

        let Ok(form_data) = FormData::new_with_form(&form) else {
            return;
        };

        if let Some(file) = poster_file.get_untracked() {
            let _ = form_data.append_with_blob_and_filename("poster_file", &file, &file.name());
        }
        let _ = form_data.append_with_str("kind", kind.as_str());
        let _ = form_data.append_with_str("id", &id.to_string());
        let _ = form_data.append_with_str(
            "remove_poster",
            if remove_poster.get_untracked() {
                "true"
            } else {
                "false"
            },
        );

        update_action.dispatch(form_data);
    };

    // Navigate back on success.
    let back_href = kind.detail_href(id);
    let navigate = use_navigate();
    Effect::new(move |_| {
        if matches!(update_action.value().get(), Some(Ok(()))) {
            navigate(&back_href, Default::default());
        }
    });

    // Revoke the blob URL when the component unmounts.
    on_cleanup(move || {
        if let Some(url) = preview_url.get_untracked() {
            let _ = Url::revoke_object_url(&url);
        }
    });

    let error_view = move || match update_action.value().get() {
        Some(Err(e)) => Some(view! {
            <div class="bg-red-500/15 text-red-300 border border-red-500/30 rounded-xl p-3 text-sm">
                {e.to_string()}
            </div>
        }),
        _ => None,
    };

    view! {
        <form node_ref=form_ref on:submit=on_submit class="space-y-5">
            <PosterEditor
                initial_poster=initial_poster
                poster_file
                preview_url
                remove_poster
            />

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

            <div>
                <label class="block text-sm font-medium text-gray-300 mb-1.5">
                    "الوصف (اختياري)"
                </label>
                <textarea
                    name="description"
                    rows=4
                    prop:value=description
                    on:input=move |ev| description.set(event_target_value(&ev))
                    placeholder="وصف مختصر..."
                    class=TEXTAREA_CLASS
                ></textarea>
            </div>

            {error_view}

            <div class="flex flex-col sm:flex-row gap-3 pt-2">
                <button
                    type="submit"
                    disabled=move || update_action.pending().get()
                    class="flex-1 py-3 px-6 rounded-2xl bg-gradient-to-r from-cyan-500 to-blue-500 hover:from-cyan-400 hover:to-blue-400 text-white font-bold shadow-lg shadow-cyan-500/20 transition-all hover:scale-[1.02] hover:shadow-cyan-500/40 disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:scale-100"
                >
                    {move || if update_action.pending().get() { "جاري الحفظ..." } else { "حفظ التغييرات" }}
                </button>
                <a
                    href=kind.detail_href(id)
                    class="flex-1 py-3 px-6 rounded-2xl bg-white/5 hover:bg-white/10 text-gray-300 font-bold text-center transition-all border border-white/10"
                >
                    "إلغاء"
                </a>
            </div>
        </form>
    }
}

#[component]
fn PosterEditor(
    initial_poster: Option<String>,
    poster_file: RwSignal<Option<web_sys::File>>,
    preview_url: RwSignal<Option<String>>,
    remove_poster: RwSignal<bool>,
) -> impl IntoView {
    let input_id = "posterEditInput";
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
        }
        poster_file.set(Some(file));
        remove_poster.set(false);
    };

    let clear_new = move |_| {
        if let Some(prev) = preview_url.get_untracked() {
            let _ = Url::revoke_object_url(&prev);
        }
        preview_url.set(None);
        file_name.set(String::new());
        poster_file.set(None);
    };

    let toggle_remove = move |_| {
        let was = remove_poster.get_untracked();
        remove_poster.set(!was);
        if !was {
            // If we're switching to "remove current", drop any staged new file.
            if let Some(prev) = preview_url.get_untracked() {
                let _ = Url::revoke_object_url(&prev);
            }
            preview_url.set(None);
            file_name.set(String::new());
            poster_file.set(None);
        }
    };

    let preview = {
        let initial_poster = initial_poster.clone();
        move || {
            if remove_poster.get() {
                Either::Left(view! {
                    <div class="w-full h-full flex items-center justify-center text-gray-600 text-xs text-center px-2">
                        "سيتم حذف الصورة"
                    </div>
                })
            } else if let Some(url) = preview_url.get() {
                Either::Right(Either::Left(view! {
                    <img src=url class="w-full h-full object-cover" alt=""/>
                }))
            } else if let Some(src) = initial_poster.clone() {
                Either::Right(Either::Right(view! {
                    <img src=src class="w-full h-full object-cover" alt=""/>
                }))
            } else {
                Either::Left(view! {
                    <div class="w-full h-full flex items-center justify-center text-gray-600 text-xs text-center px-2">
                        "لا توجد صورة"
                    </div>
                })
            }
        }
    };

    view! {
        <div>
            <label class="block text-sm font-medium text-gray-300 mb-1.5">"الصورة"</label>
            <div class="flex items-start gap-4">
                <div class="w-28 h-40 rounded-xl border border-white/10 bg-white/5 flex items-center justify-center overflow-hidden shrink-0">
                    {preview}
                </div>
                <div class="flex-1 flex flex-col gap-2 min-w-0">
                    <input
                        type="file"
                        id=input_id
                        class="hidden"
                        accept=IMAGE_ACCEPT
                        on:change=on_change
                    />
                    <div class="flex flex-wrap gap-2">
                        <label for=input_id class=UPLOAD_BTN_CLASS>
                            <UploadIcon/> "استبدال الصورة"
                        </label>
                        <button
                            type="button"
                            on:click=toggle_remove
                            class=move || format!(
                                "inline-flex items-center gap-1.5 backdrop-blur-md font-medium py-1.5 px-3 rounded-lg transition text-sm {}",
                                if remove_poster.get() {
                                    "bg-red-500/20 text-red-300 border border-red-500/30"
                                } else {
                                    "bg-white/5 hover:bg-white/10 text-gray-300 border border-white/10"
                                }
                            )
                        >
                            <DeleteIcon/>
                            {move || if remove_poster.get() { "إلغاء الحذف" } else { "حذف الصورة" }}
                        </button>
                    </div>
                    <Show when=move || !file_name.get().is_empty()>
                        <div class="flex items-center gap-2 text-xs text-gray-400">
                            <span class="truncate">{move || file_name.get()}</span>
                            <button
                                type="button"
                                on:click=clear_new
                                class="text-red-400 hover:text-red-300 transition shrink-0"
                                aria-label="إلغاء الملف المحدد"
                            >
                                <DeleteIcon/>
                            </button>
                        </div>
                    </Show>
                    <p class="text-xs text-gray-500">
                        "اترك الحقل فارغاً للإبقاء على الصورة الحالية."
                    </p>
                </div>
            </div>
        </div>
    }
}

// ─── Route wrappers ───────────────────────────────────────────────────────

macro_rules! edit_page {
    ($page:ident, $kind:expr) => {
        pub struct $page {
            id: u64,
        }

        #[lazy_route]
        impl LazyRoute for $page {
            fn data() -> Self {
                let params = use_params_map();
                let id = move || {
                    params.with(|p| p.get("id").and_then(|s| s.parse::<u64>().ok()).unwrap_or(0))
                };
                Self { id: id() }
            }

            fn view(this: Self) -> AnyView {
                view! {
                    <MetadataEditor kind=$kind id=this.id/>
                }
                .into_any()
            }
        }
    };
}

edit_page!(MovieEditPage, MetadataKind::Movie);
edit_page!(SeriesEditPage, MetadataKind::Series);
edit_page!(AudioGroupEditPage, MetadataKind::AudioGroup);
