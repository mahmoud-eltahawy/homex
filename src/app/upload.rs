use super::model::MediaType;
use crate::app::{
    icons::{
        AudioIcon, DeleteIcon, DownArrow, MovieIcon, SeriesIcon, SortIcon, UpArrow, UploadIcon,
    },
    resource_view::ResourceView,
};
use gloo_timers::future::TimeoutFuture;
use leptos::prelude::*;
use leptos::{either::Either, html};
use leptos_router::{LazyRoute, lazy_route};
use serde::{Deserialize, Serialize};
use server_fn::codec::{MultipartData, MultipartFormData};
use std::rc::Rc;
use std::time::Duration;
use web_sys::{
    FormData, HtmlFormElement, HtmlInputElement, HtmlSelectElement, MouseEvent, Url,
    wasm_bindgen::JsCast,
};

// ─── Classes ──────────────────────────────────────────────────────────────

const INPUT_CLASS: &str = "w-full bg-white/10 backdrop-blur-md text-white placeholder-gray-500 rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/20 transition";
const TEXTAREA_CLASS: &str = "w-full bg-white/10 backdrop-blur-md text-white placeholder-gray-500 rounded-xl py-3 px-4 focus:outline-none focus:ring-2 focus:ring-cyan-400/50 focus:bg-white/20 transition resize-none";
const CARD_CLASS: &str =
    "backdrop-blur-xl bg-white/5 rounded-3xl border border-white/10 p-6 md:p-8 shadow-2xl";
const ITEM_CARD_CLASS: &str = "bg-white/5 backdrop-blur-sm rounded-xl border border-white/10 p-4 flex flex-col sm:flex-row gap-3 items-start";
const TOOLBAR_BTN_CLASS: &str = "inline-flex items-center gap-1.5 bg-white/10 hover:bg-white/20 backdrop-blur-md text-white font-medium py-1.5 px-3 rounded-lg transition text-sm";
const UPLOAD_BTN_CLASS: &str = "inline-flex items-center gap-1.5 bg-green-500/20 hover:bg-green-500/30 backdrop-blur-md text-green-300 font-medium py-1.5 px-3 rounded-lg cursor-pointer transition text-sm";
const ICON_BTN_CLASS: &str = "text-gray-400 hover:text-white transition disabled:opacity-30 p-1";

// File-picker hints. Browsers treat these as suggestions — the server
// still validates the actual extension before doing anything.
const VIDEO_ACCEPT: &str = "video/mp4,video/webm,video/x-matroska,video/quicktime,\
video/x-msvideo,video/x-ms-wmv,video/x-flv,video/mp2t,\
.mp4,.m4v,.webm,.mkv,.mov,.avi,.wmv,.flv,.ts";

const AUDIO_ACCEPT: &str = "audio/mpeg,audio/mp4,audio/aac,audio/wav,audio/x-wav,\
audio/ogg,audio/opus,audio/flac,audio/x-flac,audio/x-ms-wma,audio/aiff,\
.mp3,.m4a,.aac,.wav,.ogg,.oga,.opus,.flac,.wma,.aiff,.aif";

const IMAGE_ACCEPT: &str = "image/jpeg,image/png,image/webp,.jpg,.jpeg,.png,.webp";

// ─── DTOs ─────────────────────────────────────────────────────────────────

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

// ─── Internal (server-side) types ─────────────────────────────────────────

#[cfg(feature = "ssr")]
struct UploadPayload {
    title: String,
    description: String,
    media_type: MediaType,
    is_new: bool,
    existing_id: Option<i64>,
    season_number: Option<i64>,
    files: Vec<UploadFile>,
    /// `Some((extension, bytes))` if the user attached a poster.
    poster: Option<(String, Vec<u8>)>,
}

#[cfg(feature = "ssr")]
struct UploadFile {
    filename: String,
    title: String,
    bytes: Vec<u8>,
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

#[server(input = MultipartFormData)]
pub async fn upload_media(data: MultipartData) -> Result<UploadResult, ServerFnError> {
    use crate::app::server::AppState;
    use crate::app::server::convert::{
        Job, JobPhase, is_audio_container_supported, is_convertible_audio, is_convertible_video,
        is_video_container_supported,
    };
    use std::collections::BTreeMap;

    let state: AppState = expect_context();
    let mut multipart = data.into_inner().unwrap();

    // ── Parse multipart ──────────────────────────────────────────────
    let mut title = String::new();
    let mut media_type_str = String::new();
    let mut description = String::new();
    let mut is_new = true;
    let mut existing_id: Option<i64> = None;
    let mut season_number: Option<i64> = None;

    let mut files: BTreeMap<usize, (String, Vec<u8>)> = BTreeMap::new();
    let mut file_titles: BTreeMap<usize, String> = BTreeMap::new();
    let mut poster: Option<(String, Vec<u8>)> = None;

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().map(String::from).unwrap_or_default();

        if name == "poster_file" {
            let fname = field.file_name().map(String::from).unwrap_or_default();
            let bytes = field.bytes().await?.to_vec();
            if !bytes.is_empty() {
                poster = Some((extension_of(&fname), bytes));
            }
            continue;
        }

        match name.as_str() {
            "title" => {
                title = field.text().await?;
                continue;
            }
            "media_type" => {
                media_type_str = field.text().await?;
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
                existing_id = field.text().await?.parse().ok();
                continue;
            }
            "season_number" => {
                season_number = field.text().await?.parse().ok();
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
                let name = field.file_name().map(String::from).unwrap_or_default();
                let bytes = field.bytes().await?.to_vec();
                files.insert(idx, (name, bytes));
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

    let media_type: MediaType = media_type_str
        .as_str()
        .try_into()
        .map_err(|e: &str| ServerFnError::new(e))?;

    // ── Validate extensions ──────────────────────────────────────────
    // `accept` on the file input is advisory. A user can pick "All files",
    // drag-and-drop, or curl the endpoint directly. Reject mismatches here
    // so ffmpeg never has to guess at a random blob.
    for (filename, _) in files.values() {
        let ext = filename
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        let ok = match media_type {
            MediaType::Movie | MediaType::Series => {
                is_video_container_supported(&ext) || is_convertible_video(&ext)
            }
            MediaType::AudioGroup => {
                is_audio_container_supported(&ext) || is_convertible_audio(&ext)
            }
        };
        if !ok {
            return Err(ServerFnError::new(format!("صيغة غير مدعومة: {filename}")));
        }
    }

    // ── Decide fast vs slow path ─────────────────────────────────────
    let needs_conversion = files.iter().any(|(_, (filename, _))| {
        let ext = filename.rsplit('.').next().unwrap_or("");
        match media_type {
            MediaType::Movie | MediaType::Series => !is_video_container_supported(ext),
            MediaType::AudioGroup => !is_audio_container_supported(ext),
        }
    });

    let payload = UploadPayload {
        title,
        description,
        media_type,
        is_new,
        existing_id,
        season_number,
        files: files
            .into_iter()
            .map(|(idx, (name, bytes))| {
                let t = file_titles
                    .get(&idx)
                    .cloned()
                    .unwrap_or_else(|| name.clone());
                UploadFile {
                    filename: name,
                    title: t,
                    bytes,
                }
            })
            .collect(),
        poster,
    };

    if !needs_conversion {
        // Fast path: everything inline, no job, no polling.
        let msg = process_upload(payload, &state, None).await?;
        return Ok(UploadResult {
            success: true,
            message: msg,
            job_id: None,
        });
    }

    // Slow path: spawn a background task and return immediately.
    let job_id = new_job_id();
    {
        let mut jobs = state.jobs.write().await;
        jobs.insert(
            job_id.clone(),
            Job {
                phase: JobPhase::Writing,
                started_at: std::time::Instant::now(),
            },
        );
    }

    let state_bg = state.clone();
    let job_id_bg = job_id.clone();
    tokio::spawn(async move {
        if let Err(e) = process_upload(payload, &state_bg, Some(&job_id_bg)).await {
            crate::app::server::convert::job_set_phase(
                &state_bg.jobs,
                Some(&job_id_bg),
                JobPhase::Failed(e.to_string()),
            )
            .await;
        }
    });

    Ok(UploadResult {
        success: true,
        message: "بدأ رفع الملفات وتحويلها في الخلفية".into(),
        job_id: Some(job_id),
    })
}

#[server]
pub async fn poll_conversion(job_id: String) -> Result<ConversionStatus, ServerFnError> {
    use crate::app::server::AppState;
    use crate::app::server::convert::JobPhase;

    let state: AppState = expect_context();
    let jobs = state.jobs.read().await;
    let job = jobs
        .get(&job_id)
        .ok_or_else(|| ServerFnError::new("job not found"))?;

    Ok(match &job.phase {
        JobPhase::Writing => ConversionStatus::Writing,
        JobPhase::Converting {
            conversion_index,
            conversion_count,
            current_file,
            progress,
        } => ConversionStatus::Converting {
            conversion_index: *conversion_index,
            conversion_count: *conversion_count,
            current_file: current_file.clone(),
            progress: *progress,
        },
        JobPhase::Finalizing => ConversionStatus::Finalizing,
        JobPhase::Done => ConversionStatus::Done,
        JobPhase::Failed(e) => ConversionStatus::Failed(e.clone()),
    })
}

// ─── Worker (server-side) ─────────────────────────────────────────────────

#[cfg(feature = "ssr")]
async fn process_upload(
    payload: UploadPayload,
    state: &crate::app::server::AppState,
    job_id: Option<&str>,
) -> Result<String, ServerFnError> {
    use crate::app::server::convert::{
        JobPhase, TargetFormat, convert_file, ffprobe_duration, is_audio_container_supported,
        is_video_container_supported, job_set_phase,
    };
    use sqlx::AssertSqlSafe;

    job_set_phase(&state.jobs, job_id, JobPhase::Writing).await;

    let slug = slugify(&payload.title);
    let base_subdir = match payload.media_type {
        MediaType::Movie => "movies",
        MediaType::Series => "series",
        MediaType::AudioGroup => "audio",
    };
    let base = state
        .config
        .storage
        .media_root
        .join(base_subdir)
        .join(&slug);

    tokio::fs::create_dir_all(&base)
        .await
        .map_err(|e| ServerFnError::new(format!("mkdir: {e}")))?;

    // Decide per-file: convert to what, or leave alone?
    let targets: Vec<Option<TargetFormat>> = payload
        .files
        .iter()
        .map(|f| {
            let ext = f.filename.rsplit('.').next().unwrap_or("");
            match payload.media_type {
                MediaType::Movie | MediaType::Series => {
                    if is_video_container_supported(ext) {
                        None
                    } else {
                        Some(TargetFormat::Mp4)
                    }
                }
                MediaType::AudioGroup => {
                    if is_audio_container_supported(ext) {
                        None
                    } else {
                        Some(TargetFormat::Mp3)
                    }
                }
            }
        })
        .collect();

    let conversion_count = targets.iter().filter(|t| t.is_some()).count();

    // ── Write + convert each file ────────────────────────────────────
    let mut staged: Vec<(String, i64, u64, String)> = Vec::new();

    for (i, file) in payload.files.iter().enumerate() {
        let ext = file.filename.rsplit('.').next().unwrap_or("").to_string();
        let stem = file
            .filename
            .rsplitn(2, '.')
            .nth(1)
            .unwrap_or(&file.filename)
            .to_string();
        let safe_stem = sanitize_filename(&stem);

        let raw_rel = format!("{base_subdir}/{slug}/{safe_stem}.{ext}");
        let raw_abs = state.config.storage.media_root.join(&raw_rel);

        tokio::fs::write(&raw_abs, &file.bytes)
            .await
            .map_err(|e| ServerFnError::new(format!("write {}: {e}", raw_abs.display())))?;

        let Some(target) = targets[i] else {
            // Container is already playable.
            let dur = ffprobe_duration(&raw_abs).await.round() as i64;
            staged.push((raw_rel, dur, file.bytes.len() as u64, file.title.clone()));
            continue;
        };

        let out_rel = format!("{base_subdir}/{slug}/{safe_stem}.{}", target.extension());
        let out_abs = state.config.storage.media_root.join(&out_rel);

        let conversion_pos = targets[..i].iter().filter(|t| t.is_some()).count();
        let total_secs = ffprobe_duration(&raw_abs).await;

        convert_file(
            &raw_abs,
            &out_abs,
            target,
            total_secs,
            conversion_pos,
            conversion_count,
            &file.filename,
            &state.jobs,
            job_id,
        )
        .await
        .map_err(ServerFnError::new)?;

        let _ = tokio::fs::remove_file(&raw_abs).await;

        let size = tokio::fs::metadata(&out_abs)
            .await
            .map(|m| m.len())
            .unwrap_or(0);
        staged.push((out_rel, total_secs.round() as i64, size, file.title.clone()));
    }

    // ── DB inserts ───────────────────────────────────────────────────
    job_set_phase(&state.jobs, job_id, JobPhase::Finalizing).await;

    let mut file_rows: Vec<(i64, String)> = Vec::new();
    for (rel, dur, size, title) in &staged {
        let file_id: i64 = sqlx::query_scalar(
            "INSERT INTO files (relative_path, size_bytes, duration_secs) \
             VALUES (?, ?, ?) RETURNING id",
        )
        .bind(rel)
        .bind(*size as i64)
        .bind(*dur)
        .fetch_one(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

        file_rows.push((file_id, title.clone()));
    }

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // The match returns (table_name, entity_id) so poster handling can
    // run once afterwards, regardless of media type.
    let (table, entity_id): (&str, i64) = match payload.media_type {
        MediaType::Movie => {
            let movie_id: i64 = if payload.is_new {
                sqlx::query_scalar(
                    "INSERT INTO movies (title, description) VALUES (?, ?) RETURNING id",
                )
                .bind(&payload.title)
                .bind(if payload.description.is_empty() {
                    None
                } else {
                    Some(&payload.description)
                })
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?
            } else {
                payload
                    .existing_id
                    .ok_or_else(|| ServerFnError::new("existing_id required"))?
            };

            let start: i64 = sqlx::query_scalar(
                "SELECT COALESCE(MAX(number) + 1, 0) FROM movie_chapters WHERE movie_id = ?",
            )
            .bind(movie_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

            for (i, (fid, title)) in file_rows.iter().enumerate() {
                sqlx::query(
                    "INSERT INTO movie_chapters (movie_id, number, title, file_id) \
                     VALUES (?, ?, ?, ?)",
                )
                .bind(movie_id)
                .bind(start + i as i64)
                .bind(title)
                .bind(fid)
                .execute(&mut *tx)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;
            }
            ("movies", movie_id)
        }
        MediaType::Series => {
            let series_id: i64 = if payload.is_new {
                sqlx::query_scalar(
                    "INSERT INTO series (title, description) VALUES (?, ?) RETURNING id",
                )
                .bind(&payload.title)
                .bind(if payload.description.is_empty() {
                    None
                } else {
                    Some(&payload.description)
                })
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?
            } else {
                payload
                    .existing_id
                    .ok_or_else(|| ServerFnError::new("existing_id required"))?
            };

            let sn = payload.season_number.unwrap_or(1);
            let season_id: i64 = sqlx::query_scalar(
                "INSERT INTO seasons (series_id, number) VALUES (?, ?) \
                 ON CONFLICT(series_id, number) DO UPDATE SET number = number \
                 RETURNING id",
            )
            .bind(series_id)
            .bind(sn)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

            let start: i64 = sqlx::query_scalar(
                "SELECT COALESCE(MAX(number) + 1, 1) FROM episodes WHERE season_id = ?",
            )
            .bind(season_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

            for (i, (fid, title)) in file_rows.iter().enumerate() {
                sqlx::query(
                    "INSERT INTO episodes (season_id, number, title, file_id) \
                     VALUES (?, ?, ?, ?)",
                )
                .bind(season_id)
                .bind(start + i as i64)
                .bind(title)
                .bind(fid)
                .execute(&mut *tx)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;
            }
            ("series", series_id)
        }
        MediaType::AudioGroup => {
            let group_id: i64 = if payload.is_new {
                sqlx::query_scalar(
                    "INSERT INTO audio_groups (title, description) VALUES (?, ?) RETURNING id",
                )
                .bind(&payload.title)
                .bind(if payload.description.is_empty() {
                    None
                } else {
                    Some(&payload.description)
                })
                .fetch_one(&mut *tx)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?
            } else {
                payload
                    .existing_id
                    .ok_or_else(|| ServerFnError::new("existing_id required"))?
            };

            let start: i64 = sqlx::query_scalar(
                "SELECT COALESCE(MAX(number) + 1, 0) FROM audios WHERE group_id = ?",
            )
            .bind(group_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

            for (i, (fid, title)) in file_rows.iter().enumerate() {
                sqlx::query(
                    "INSERT INTO audios (group_id, number, title, file_id) \
                     VALUES (?, ?, ?, ?)",
                )
                .bind(group_id)
                .bind(start + i as i64)
                .bind(title)
                .bind(fid)
                .execute(&mut *tx)
                .await
                .map_err(|e| ServerFnError::new(e.to_string()))?;
            }
            ("audio_groups", group_id)
        }
    };

    // ── Poster (optional) ────────────────────────────────────────────
    // Written with the entity id in the filename so two uploads with the
    // same title (slug) can't fight over the same file.
    if let Some((ext, bytes)) = payload.poster.as_ref() {
        let filename = format!("{entity_id}.{ext}");
        let rel_dir = base_subdir.to_string();
        let abs_dir = state.config.storage.data_dir.join("posters").join(&rel_dir);
        tokio::fs::create_dir_all(&abs_dir)
            .await
            .map_err(|e| ServerFnError::new(format!("mkdir posters: {e}")))?;

        let abs = abs_dir.join(&filename);
        tokio::fs::write(&abs, bytes)
            .await
            .map_err(|e| ServerFnError::new(format!("write poster: {e}")))?;

        let url = format!("/posters/{rel_dir}/{filename}");
        let sql = AssertSqlSafe(format!("UPDATE {table} SET poster = ? WHERE id = ?"));
        sqlx::query(sql)
            .bind(&url)
            .bind(entity_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;
    }

    tx.commit()
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    job_set_phase(&state.jobs, job_id, JobPhase::Done).await;

    Ok(format!("تم رفع {} ملف بنجاح", file_rows.len()))
}

// ─── Server-only helpers ──────────────────────────────────────────────────

#[cfg(feature = "ssr")]
fn slugify(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut last_dash = false;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash && !out.is_empty() {
            out.push('-');
            last_dash = true;
        }
    }
    out.trim_matches('-').to_string()
}

#[cfg(feature = "ssr")]
fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| !matches!(c, '/' | '\\' | '\0'))
        .collect()
}

#[cfg(feature = "ssr")]
fn extension_of(filename: &str) -> String {
    match filename.rsplit_once('.') {
        Some((_, ext))
            if !ext.is_empty()
                && ext.len() <= 8
                && ext.chars().all(|c| c.is_ascii_alphanumeric()) =>
        {
            ext.to_ascii_lowercase()
        }
        _ => "jpg".to_string(),
    }
}

#[cfg(feature = "ssr")]
fn new_job_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{ns:x}")
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

    let title = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let season_number = RwSignal::new(1u32);
    let poster_file = RwSignal::new(None::<web_sys::File>);

    let items = RwSignal::new(Vec::<UploadItem>::new());
    let next_id = RwSignal::new(1u32);

    // Background job tracking.
    let active_job = RwSignal::new(None::<String>);
    let job_status = RwSignal::new(None::<ConversionStatus>);

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

    let s2_ref = NodeRef::<html::Div>::new();
    let s3_ref = NodeRef::<html::Div>::new();

    Effect::new(move |prev: Option<bool>| {
        let curr = media_type.get().is_some();
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
        let curr = is_new.get().is_some();
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

    // When the server hands back a job_id, start polling.
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

        // Poster (optional)
        if let Some(poster) = poster_file.get_untracked() {
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
        Some(Ok(_)) => None, // will be shown as ConversionProgress
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
                done=Signal::derive(move || media_type.get().is_some())
            >
                <MediaKindCards
                    media_type
                    is_new
                    existing_id
                    title
                    description
                    season_number
                    poster_file
                    items
                    next_id
                />
            </FormSection>

            // ② New vs existing
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

            // ③ Details
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
                            poster_file
                            titles
                        />
                    </FormSection>
                </div>
            </Show>

            // ④ Files
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
                                hint="الفيديوهات بصيغة غير مدعومة (مثل MKV) سيتم تحويلها إلى MP4 تلقائياً."
                                input_id="multiMovieInput"
                                accept=VIDEO_ACCEPT
                                select_label="اختيار فصول الفيلم"
                                number_label="رقم الفصل"
                                title_label="عنوان الفصل"
                                file_label="الملف"
                                icon=MovieIcon()
                            />
                        }.into_any(),
                        Some(MediaType::Series) => view! {
                            <MediaFilesSection
                                items
                                next_id
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
                        }.into_any(),
                        Some(MediaType::AudioGroup) => view! {
                            <MediaFilesSection
                                items
                                next_id
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
                        }.into_any(),
                        None => ().into_any(),
                    }}
                </FormSection>
            </Show>

            <HiddenFormState media_type is_new existing_id/>

            {result_view}

            <Show when=move || job_status.get().is_some()>
                {move || job_status.get().map(|s| view! { <ConversionProgress status=s/> })}
            </Show>

            <UploadSubmitButton
                pending=Signal::derive(move || {
                    upload_action.pending().get() || active_job.get().is_some()
                })
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

// ─── Section ① ────────────────────────────────────────────────────────────

#[component]
#[allow(clippy::too_many_arguments)]
fn MediaKindCards(
    media_type: RwSignal<Option<MediaType>>,
    is_new: RwSignal<Option<bool>>,
    existing_id: RwSignal<Option<i64>>,
    title: RwSignal<String>,
    description: RwSignal<String>,
    season_number: RwSignal<u32>,
    poster_file: RwSignal<Option<web_sys::File>>,
    items: RwSignal<Vec<UploadItem>>,
    next_id: RwSignal<u32>,
) -> impl IntoView {
    let on_select: Rc<dyn Fn(MediaType)> = Rc::new(move |mt: MediaType| {
        if media_type.get_untracked() == Some(mt) {
            return;
        }
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
            poster_file.set(None);
            items.set(Vec::new());
            next_id.set(1);
        });
    });

    view! {
        <div class="grid grid-cols-3 gap-3">
            <MediaKindCard value=MediaType::Movie label="فيلم" icon=MovieIcon()
                media_type on_select=Rc::clone(&on_select)/>
            <MediaKindCard value=MediaType::Series label="مسلسل" icon=SeriesIcon()
                media_type on_select=Rc::clone(&on_select)/>
            <MediaKindCard value=MediaType::AudioGroup label="صوتيات" icon=AudioIcon()
                media_type on_select=on_select/>
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
    media_type: RwSignal<Option<MediaType>>,
    is_new: RwSignal<Option<bool>>,
    existing_id: RwSignal<Option<i64>>,
    title: RwSignal<String>,
    description: RwSignal<String>,
    season_number: RwSignal<u32>,
    poster_file: RwSignal<Option<web_sys::File>>,
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
                <ExistingSelectField media_type=media_type.get() existing_id titles/>
            </Show>

            <PosterUploadField poster_file/>

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
            <input type="text" name="title"
                prop:value=title
                on:input=move |ev| title.set(event_target_value(&ev))
                placeholder="أدخل العنوان..." class=INPUT_CLASS/>
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
            <textarea name="description" rows=3
                prop:value=description
                on:input=move |ev| description.set(event_target_value(&ev))
                placeholder="وصف مختصر (اختياري)..." class=TEXTAREA_CLASS
            ></textarea>
        </div>
    }
}

#[component]
fn SeasonNumberField(season_number: RwSignal<u32>) -> impl IntoView {
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
fn PosterUploadField(poster_file: RwSignal<Option<web_sys::File>>) -> impl IntoView {
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

// ─── Section ④ ────────────────────────────────────────────────────────────

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

// ─── Hidden form state ────────────────────────────────────────────────────

#[component]
fn HiddenFormState(
    media_type: RwSignal<Option<MediaType>>,
    is_new: RwSignal<Option<bool>>,
    existing_id: RwSignal<Option<i64>>,
) -> impl IntoView {
    view! {
        <input type="hidden" name="media_type"
            value=move || media_type.get().map(|m| m.to_string()).unwrap_or_default()/>
        <input type="hidden" name="is_new"
            value=move || is_new.get().map(|b| b.to_string()).unwrap_or_default()/>
        <input type="hidden" name="existing_id"
            value=move || existing_id.get().map(|id| id.to_string()).unwrap_or_default()/>
    }
}

// ─── Header, progress, submit ────────────────────────────────────────────

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
fn ConversionProgress(status: ConversionStatus) -> impl IntoView {
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
fn UploadSubmitButton(
    pending: Signal<bool>,
    valid: Signal<bool>,
    hint: Signal<String>,
    file_count: Signal<usize>,
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
                        if n == 0 { "رفع الوسائط".to_string() }
                        else { format!("رفع الوسائط ({n})") }
                    }
                }}
            </button>
            <p class="text-center text-xs text-gray-500 h-4 leading-4">
                {move || hint.get()}
            </p>
        </div>
    }
}
