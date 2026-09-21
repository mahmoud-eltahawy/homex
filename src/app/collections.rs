use crate::app::model::{Collection, Item};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::app::server::{
    SqlErr,
    db::{self, CollectionField},
};

#[server]
pub async fn fetch_collections(
    section_slug: String,
    offset: usize,
    size: usize,
    search_query: Option<String>,
) -> Result<Vec<Collection>, ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    db::fetch_collections(
        &state.db,
        &section_slug,
        offset,
        size,
        search_query.as_deref(),
    )
    .await
    .srv()
}

#[server]
pub async fn fetch_collections_count(
    section_slug: String,
    search_query: Option<String>,
) -> Result<usize, ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    let n = db::fetch_collections_count(&state.db, &section_slug, search_query.as_deref())
        .await
        .srv()?;
    Ok(n as usize)
}

#[server]
pub async fn fetch_collection_detail(
    section_slug: String,
    collection_id: u64,
) -> Result<Collection, ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    db::fetch_collection_detail(&state.db, &section_slug, collection_id as i64)
        .await
        .srv()?
        .ok_or_else(|| ServerFnError::new("collection not found"))
}

#[server]
pub async fn fetch_items(collection_id: u64) -> Result<Vec<Item>, ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    db::fetch_items(&state.db, collection_id as i64).await.srv()
}

#[server]
pub async fn create_empty_collection(section_slug: String) -> Result<u64, ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    let id = db::insert_empty_collection(&state.db, &section_slug)
        .await
        .srv()?;
    Ok(id as u64)
}

#[server]
pub async fn patch_collection_field(
    id: u64,
    field: String,
    value: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    let f = CollectionField::parse(&field).ok_or_else(|| ServerFnError::new("حقل غير مسموح"))?;
    db::update_collection_field(&state.db, id as i64, f, value.as_deref())
        .await
        .srv()?;
    Ok(())
}

#[server]
pub async fn patch_item_title(id: u64, title: String) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    let t = if title.trim().is_empty() {
        None
    } else {
        Some(title)
    };
    db::update_item_title(&state.db, id as i64, t.as_deref())
        .await
        .srv()?;
    Ok(())
}

#[server]
pub async fn delete_collection(id: u64) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    db::delete_collection(&state.db, id as i64).await.srv()?;
    Ok(())
}

#[server]
pub async fn delete_item(id: u64) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    db::delete_item(&state.db, id as i64).await.srv()?;
    Ok(())
}

#[server(input = server_fn::codec::MultipartFormData)]
pub async fn upload_collection_poster(
    data: server_fn::codec::MultipartData,
) -> Result<String, ServerFnError> {
    use crate::app::server::{AppState, poster::write_poster, upload::extension_of};
    let state: AppState = expect_context();

    let mut mp = data.into_inner().unwrap();
    let mut section_slug = String::new();
    let mut id: i64 = 0;
    let mut bytes: Option<(String, Vec<u8>)> = None;

    while let Some(f) = mp.next_field().await? {
        match f.name().unwrap_or("") {
            "section" => section_slug = f.text().await?,
            "id" => id = f.text().await?.parse().unwrap_or(0),
            "poster_file" => {
                let fname = f.file_name().unwrap_or("").to_string();
                let b = f.bytes().await?.to_vec();
                if !b.is_empty() {
                    bytes = Some((extension_of(&fname), b));
                }
            }
            _ => {
                let _ = f.bytes().await?;
            }
        }
    }

    let (ext, bytes) = bytes.ok_or_else(|| ServerFnError::new("لم يتم استلام صورة"))?;
    let url = write_poster(
        &state.config.storage.data_dir,
        &section_slug,
        id,
        &ext,
        &bytes,
    )
    .await?;

    db::update_collection_field(&state.db, id, CollectionField::Poster, Some(&url))
        .await
        .srv()?;
    Ok(url)
}
