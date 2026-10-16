use std::path::{Path, PathBuf};

use server_fn::codec::MultipartData;
use tokio::io::AsyncWriteExt;

use crate::app::error::{AppError, ErrorCode};
use crate::app::types::{FileExt, FileName};
use crate::app::upload_types::{FILE_FIELD, FORM_FIELD, POSTER_FIELD, UploadForm};

use super::naming::extension_of;
use super::types::{UploadFile, UploadPayload};

fn fresh_temp_dir() -> Result<tempfile::TempDir, AppError> {
    tempfile::Builder::new()
        .prefix("homex-upload-")
        .tempdir()
        .map_err(|e| AppError::internal("create upload temp dir", e))
}

async fn stream_field_to_file(field: &mut multer::Field<'_>, dest: &Path) -> Result<u64, AppError> {
    let mut out = tokio::fs::File::create(dest)
        .await
        .map_err(|e| AppError::internal("create upload temp file", e))?;
    let mut size: u64 = 0;
    while let Some(chunk) = field.chunk().await? {
        out.write_all(&chunk)
            .await
            .map_err(|e| AppError::internal("write upload temp file", e))?;
        size += chunk.len() as u64;
    }
    out.flush().await.ok();
    Ok(size)
}

async fn drain_field(mut field: multer::Field<'_>) -> Result<(), AppError> {
    while field.chunk().await?.is_some() {}
    Ok(())
}

/// Parse a media-upload multipart request: one `form` JSON field followed
/// by zero or more raw `file` fields and optionally one `poster_file`.
pub async fn parse_upload_multipart(data: MultipartData) -> Result<UploadPayload, AppError> {
    let mut mp = data
        .into_inner()
        .ok_or_else(|| AppError::internal("multipart", "missing multipart body"))?;
    let temp_dir = fresh_temp_dir()?;

    let mut form: Option<UploadForm> = None;
    let mut files: Vec<(String, PathBuf, u64)> = Vec::new();
    let mut poster: Option<(FileExt, Vec<u8>)> = None;

    while let Some(mut field) = mp.next_field().await? {
        match field.name() {
            Some(FORM_FIELD) => {
                form = Some(serde_json::from_str(&field.text().await?).map_err(|e| {
                    AppError::hidden(ErrorCode::InvalidPath, "upload form parse", e)
                })?);
            }
            Some(POSTER_FIELD) => {
                let fname = field.file_name().unwrap_or("").to_string();
                let bytes = field.bytes().await?.to_vec();
                if !bytes.is_empty() {
                    poster = Some((extension_of(&fname), bytes));
                }
            }
            Some(FILE_FIELD) => {
                let fname = field.file_name().unwrap_or("").to_string();
                let idx = files.len();
                let path = temp_dir.path().join(format!("{idx}.bin"));
                let size = stream_field_to_file(&mut field, &path).await?;
                files.push((fname, path, size));
            }
            _ => drain_field(field).await?,
        }
    }

    let form = form.ok_or_else(|| AppError::internal("multipart", "missing form"))?;

    if form.files.len() != files.len() {
        return Err(AppError::internal(
            "multipart",
            format!(
                "form declares {} files, stream carried {}",
                form.files.len(),
                files.len()
            ),
        ));
    }

    let upload_files: Vec<UploadFile> = form
        .files
        .into_iter()
        .zip(files)
        .map(|(meta, (filename, temp_path, size))| {
            let filename = FileName::new(filename)
                .map_err(|e| AppError::hidden(ErrorCode::InvalidPath, "upload filename", e))?;
            Ok::<UploadFile, AppError>(UploadFile {
                filename,
                title: meta.title,
                temp_path,
                size,
            })
        })
        .collect::<Result<Vec<_>, AppError>>()?;

    if upload_files.is_empty() {
        return Err(AppError::public(ErrorCode::NoFilesReceived));
    }

    Ok(UploadPayload {
        section_slug: form.section_slug,
        target: form.target,
        season_number: form.season_number,
        files: upload_files,
        poster,
        temp_dir,
    })
}
