use crate::app::constants::{messages, storage};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use leptos::prelude::ServerFnError;
use server_fn::codec::MultipartData;
use tokio::io::AsyncWriteExt;

use super::naming::{extension_of, new_job_id};
use super::types::{UploadFile, UploadPayload};

// ─── Multipart form field names (server-side) ─────────────────────────────
//
// These MUST stay in sync with the field names built on the client
// (`app/collection_detail.rs`).

mod fields;

// ─── Temp dir management ──────────────────────────────────────────────────

fn fresh_temp_dir() -> PathBuf {
    std::env::temp_dir()
        .join(storage::UPLOAD_TEMP_DIR)
        .join(new_job_id())
}

async fn create_temp_dir(dir: &Path) -> Result<(), ServerFnError> {
    tokio::fs::create_dir_all(dir)
        .await
        .map_err(|e| ServerFnError::new(format!("mkdir temp: {e}")))
}

async fn wipe_temp_dir(dir: &Path) {
    let _ = tokio::fs::remove_dir_all(dir).await;
}

// ─── Accumulator ──────────────────────────────────────────────────────────

struct StagedTempFile {
    filename: String,
    temp_path: PathBuf,
    size: u64,
}

#[derive(Default)]
struct Accumulator {
    title: String,
    section_slug: String,
    description: String,
    collection_id: i64,
    season_number: Option<i64>,
    files: BTreeMap<usize, StagedTempFile>,
    file_titles: BTreeMap<usize, String>,
    poster: Option<(String, Vec<u8>)>,
}

// ─── Field classification ─────────────────────────────────────────────────

enum FieldKind {
    Poster,
    Title,
    SectionSlug,
    Description,
    CollectionId,
    SeasonNumber,
    FileTitle(usize),
    File(usize),
    Unknown,
}

fn classify_field(name: &str) -> FieldKind {
    if name == fields::POSTER_FILE {
        return FieldKind::Poster;
    }
    if let Some(idx) = name
        .strip_prefix(fields::FILE_TITLE_PREFIX)
        .and_then(|s| s.parse().ok())
    {
        return FieldKind::FileTitle(idx);
    }
    if let Some(idx) = name
        .strip_prefix(fields::FILE_PREFIX)
        .and_then(|s| s.parse().ok())
    {
        return FieldKind::File(idx);
    }
    match name {
        fields::TITLE => FieldKind::Title,
        fields::SECTION_SLUG => FieldKind::SectionSlug,
        fields::DESCRIPTION => FieldKind::Description,
        fields::COLLECTION_ID => FieldKind::CollectionId,
        fields::SEASON_NUMBER => FieldKind::SeasonNumber,
        _ => FieldKind::Unknown,
    }
}

// ─── Per-kind handlers ────────────────────────────────────────────────────

async fn handle_poster_field(
    acc: &mut Accumulator,
    field: multer::Field<'_>,
) -> Result<(), ServerFnError> {
    let fname = field.file_name().map(String::from).unwrap_or_default();
    let bytes = field.bytes().await?.to_vec();
    if !bytes.is_empty() {
        acc.poster = Some((extension_of(&fname), bytes));
    }
    Ok(())
}

async fn handle_text_field(
    dest: &mut String,
    field: multer::Field<'_>,
) -> Result<(), ServerFnError> {
    *dest = field.text().await?;
    Ok(())
}

async fn handle_i64_field(dest: &mut i64, field: multer::Field<'_>) -> Result<(), ServerFnError> {
    *dest = field.text().await?.parse()?;
    Ok(())
}

async fn handle_optional_i64_field(
    dest: &mut Option<i64>,
    field: multer::Field<'_>,
) -> Result<(), ServerFnError> {
    *dest = field.text().await?.parse().ok();
    Ok(())
}

async fn handle_file_title_field(
    acc: &mut Accumulator,
    idx: usize,
    field: multer::Field<'_>,
) -> Result<(), ServerFnError> {
    acc.file_titles.insert(idx, field.text().await?);
    Ok(())
}

async fn handle_file_field(
    acc: &mut Accumulator,
    temp_dir: &Path,
    idx: usize,
    mut field: multer::Field<'_>,
) -> Result<(), ServerFnError> {
    let fname = field.file_name().map(String::from).unwrap_or_default();
    let temp_path = temp_dir.join(format!("{idx}.bin"));
    let size = stream_field_to_file(&mut field, &temp_path).await?;
    acc.files.insert(
        idx,
        StagedTempFile {
            filename: fname,
            temp_path,
            size,
        },
    );
    Ok(())
}

async fn stream_field_to_file(
    field: &mut multer::Field<'_>,
    dest: &Path,
) -> Result<u64, ServerFnError> {
    let mut out = tokio::fs::File::create(dest)
        .await
        .map_err(|e| ServerFnError::new(format!("create temp: {e}")))?;
    let mut size: u64 = 0;
    while let Some(chunk) = field.chunk().await? {
        out.write_all(&chunk)
            .await
            .map_err(|e| ServerFnError::new(format!("write temp: {e}")))?;
        size += chunk.len() as u64;
    }
    out.flush().await.ok();
    Ok(size)
}

async fn drain_field(mut field: multer::Field<'_>) -> Result<(), ServerFnError> {
    while field.chunk().await?.is_some() {}
    Ok(())
}

// ─── Dispatcher ───────────────────────────────────────────────────────────

async fn handle_field(
    acc: &mut Accumulator,
    temp_dir: &Path,
    field: multer::Field<'_>,
) -> Result<(), ServerFnError> {
    let name = field.name().map(String::from).unwrap_or_default();
    match classify_field(&name) {
        FieldKind::Poster => handle_poster_field(acc, field).await,
        FieldKind::Title => handle_text_field(&mut acc.title, field).await,
        FieldKind::SectionSlug => handle_text_field(&mut acc.section_slug, field).await,
        FieldKind::Description => handle_text_field(&mut acc.description, field).await,
        FieldKind::CollectionId => handle_i64_field(&mut acc.collection_id, field).await,
        FieldKind::SeasonNumber => handle_optional_i64_field(&mut acc.season_number, field).await,
        FieldKind::FileTitle(idx) => handle_file_title_field(acc, idx, field).await,
        FieldKind::File(idx) => handle_file_field(acc, temp_dir, idx, field).await,
        FieldKind::Unknown => drain_field(field).await,
    }
}

// ─── Parse loop ───────────────────────────────────────────────────────────

async fn run_parse_loop(
    multipart: &mut multer::Multipart<'static>,
    temp_dir: &Path,
) -> Result<Accumulator, ServerFnError> {
    let mut acc = Accumulator::default();
    while let Some(field) = multipart.next_field().await? {
        handle_field(&mut acc, temp_dir, field).await?;
    }
    Ok(acc)
}

// ─── Finalization ─────────────────────────────────────────────────────────

fn build_upload_files(
    files: BTreeMap<usize, StagedTempFile>,
    mut titles: BTreeMap<usize, String>,
) -> Vec<UploadFile> {
    files
        .into_iter()
        .map(|(idx, sf)| {
            let title = titles.remove(&idx).unwrap_or_else(|| sf.filename.clone());
            UploadFile {
                filename: sf.filename,
                title,
                temp_path: sf.temp_path,
                size: sf.size,
            }
        })
        .collect()
}

fn finalize(acc: Accumulator, temp_dir: PathBuf) -> Result<UploadPayload, ServerFnError> {
    if acc.files.is_empty() {
        return Err(ServerFnError::new(messages::NO_FILES_RECEIVED));
    }
    if acc.section_slug.is_empty() {
        return Err(ServerFnError::new(messages::SECTION_SLUG_MISSING));
    }
    let files = build_upload_files(acc.files, acc.file_titles);
    Ok(UploadPayload {
        section_slug: acc.section_slug,
        collection_id: acc.collection_id,
        title: acc.title,
        description: acc.description,
        season_number: acc.season_number,
        files,
        poster: acc.poster,
        temp_dir,
    })
}

// ─── Public entry point ───────────────────────────────────────────────────

pub async fn parse_upload_multipart(data: MultipartData) -> Result<UploadPayload, ServerFnError> {
    let mut multipart = data.into_inner().unwrap();
    let temp_dir = fresh_temp_dir();
    create_temp_dir(&temp_dir).await?;

    let acc = match run_parse_loop(&mut multipart, &temp_dir).await {
        Ok(a) => a,
        Err(e) => {
            wipe_temp_dir(&temp_dir).await;
            return Err(e);
        }
    };

    match finalize(acc, temp_dir.clone()) {
        Ok(p) => Ok(p),
        Err(e) => {
            wipe_temp_dir(&temp_dir).await;
            Err(e)
        }
    }
}
