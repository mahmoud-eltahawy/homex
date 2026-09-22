use super::naming::extension_of;
use super::types::{UploadFile, UploadPayload};
use leptos::prelude::ServerFnError;
use server_fn::codec::MultipartData;
use std::collections::BTreeMap;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;

pub async fn parse_upload_multipart(data: MultipartData) -> Result<UploadPayload, ServerFnError> {
    let mut multipart = data.into_inner().unwrap();

    let temp_dir = std::env::temp_dir()
        .join("homex-uploads")
        .join(super::new_job_id());
    tokio::fs::create_dir_all(&temp_dir)
        .await
        .map_err(|e| ServerFnError::new(format!("mkdir temp: {e}")))?;

    let mut title = String::new();
    let mut section_slug = String::new();
    let mut description = String::new();
    let mut collection_id: i64 = 0;
    let mut season_number: Option<i64> = None;

    // idx -> (filename, temp_path, size)
    let mut files: BTreeMap<usize, (String, PathBuf, u64)> = BTreeMap::new();
    let mut file_titles: BTreeMap<usize, String> = BTreeMap::new();
    let mut poster: Option<(String, Vec<u8>)> = None;

    // Inner result so a mid-stream error still runs cleanup.
    let build: Result<(), ServerFnError> = async {
        while let Some(mut field) = multipart.next_field().await? {
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
                "section_slug" => {
                    section_slug = field.text().await?;
                    continue;
                }
                "description" => {
                    description = field.text().await?;
                    continue;
                }
                "collection_id" => {
                    collection_id = field.text().await?.parse()?;
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
                    while field.chunk().await?.is_some() {}
                }
            } else if let Some(idx_str) = name.strip_prefix("file_") {
                if let Ok(idx) = idx_str.parse::<usize>() {
                    let fname = field.file_name().map(String::from).unwrap_or_default();
                    let temp_path = temp_dir.join(format!("{idx}.bin"));

                    let mut out = tokio::fs::File::create(&temp_path)
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

                    files.insert(idx, (fname, temp_path, size));
                } else {
                    while field.chunk().await?.is_some() {}
                }
            } else {
                while field.chunk().await?.is_some() {}
            }
        }
        Ok(())
    }
    .await;

    if let Err(e) = build {
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
        return Err(e);
    }
    if files.is_empty() {
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
        return Err(ServerFnError::new("لم يتم استلام أي ملف"));
    }
    if section_slug.is_empty() {
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
        return Err(ServerFnError::new("section_slug مفقود"));
    }

    let files = files
        .into_iter()
        .map(|(idx, (filename, temp_path, size))| {
            let title = file_titles.remove(&idx).unwrap_or_else(|| filename.clone());
            UploadFile {
                filename,
                title,
                temp_path,
                size,
            }
        })
        .collect();

    Ok(UploadPayload {
        section_slug,
        collection_id,
        title,
        description,
        season_number,
        files,
        poster,
        temp_dir,
    })
}
