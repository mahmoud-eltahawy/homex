use super::naming::extension_of;
use super::types::{UploadFile, UploadPayload};
use leptos::prelude::ServerFnError;
use server_fn::codec::MultipartData;
use std::collections::BTreeMap;

pub async fn parse_upload_multipart(data: MultipartData) -> Result<UploadPayload, ServerFnError> {
    let mut multipart = data.into_inner().unwrap();

    let mut title = String::new();
    let mut section_slug = String::new();
    let mut description = String::new();
    let mut collection_id: Option<i64> = None;
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
            "section_slug" => {
                section_slug = field.text().await?;
                continue;
            }
            "description" => {
                description = field.text().await?;
                continue;
            }
            "collection_id" => {
                collection_id = field.text().await?.parse().ok();
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
                let fname = field.file_name().map(String::from).unwrap_or_default();
                let bytes = field.bytes().await?.to_vec();
                files.insert(idx, (fname, bytes));
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
    if section_slug.is_empty() {
        return Err(ServerFnError::new("section_slug مفقود"));
    }

    let files = files
        .into_iter()
        .map(|(idx, (filename, bytes))| {
            let title = file_titles.remove(&idx).unwrap_or_else(|| filename.clone());
            UploadFile {
                filename,
                title,
                bytes,
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
    })
}
