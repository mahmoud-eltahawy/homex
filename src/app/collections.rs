use crate::app::model::{Collection, CollectionSummary, Item};
use leptos::prelude::*;
use uuid::Uuid;

#[cfg(feature = "ssr")]
use crate::app::constants::messages;

#[cfg(feature = "ssr")]
use crate::app::server::{ToastyErr, db};

// ─── Multipart form field names (poster upload) ───────────────────────────

#[cfg(feature = "ssr")]
mod fields {
    pub const SECTION: &str = "section";
    pub const ID: &str = "id";
    pub const POSTER_FILE: &str = "poster_file";
}

#[server]
pub async fn fetch_collections(
    section_slug: String,
    offset: usize,
    size: usize,
    search_query: Option<String>,
) -> Result<Vec<Collection>, ServerFnError> {
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    db::fetch_collections(
        &mut state.db,
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
    let mut state: AppState = expect_context();
    let n = db::fetch_collections_count(&mut state.db, &section_slug, search_query.as_deref())
        .await
        .srv()?;
    Ok(n as usize)
}

#[server]
pub async fn fetch_collection_detail(
    section_slug: String,
    collection_id: Uuid,
) -> Result<Collection, ServerFnError> {
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    db::fetch_collection_detail(&mut state.db, &section_slug, collection_id)
        .await
        .srv()?
        .ok_or_else(|| ServerFnError::new(messages::COLLECTION_NOT_FOUND))
}

#[server]
pub async fn fetch_items(collection_id: Uuid) -> Result<Vec<Item>, ServerFnError> {
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    db::fetch_items(&mut state.db, collection_id).await.srv()
}

#[server]
pub async fn create_empty_collection(section_slug: String) -> Result<Uuid, ServerFnError> {
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    db::insert_empty_collection(&mut state.db, &section_slug)
        .await
        .srv()
}

#[server]
pub async fn patch_collection_field(
    id: Uuid,
    field: String,
    value: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    use crate::app::server::db::CollectionField;

    let mut state: AppState = expect_context();

    if field == "title" && value.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(ServerFnError::new(messages::TITLE_REQUIRED));
    }

    let f = match field.as_str() {
        "title" => CollectionField::Title,
        "description" => CollectionField::Description,
        "poster" => CollectionField::Poster,
        _ => return Err(ServerFnError::new(messages::UNKNOWN_FIELD)),
    };

    db::update_collection_field(&mut state.db, id, f, value.as_deref())
        .await
        .srv()?;
    Ok(())
}

#[server]
pub async fn patch_collection_tags(id: Uuid, tags: Vec<String>) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    db::set_collection_tags(&mut state.db, id, &tags)
        .await
        .srv()?;
    Ok(())
}

#[server]
pub async fn patch_item_title(id: Uuid, title: String) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    let t = if title.trim().is_empty() {
        None
    } else {
        Some(title)
    };
    db::update_item_title(&mut state.db, id, t.as_deref())
        .await
        .srv()?;
    Ok(())
}

#[server]
pub async fn delete_collection(id: Uuid) -> Result<(), ServerFnError> {
    use crate::app::server::{AppState, remove_media_files, remove_poster};
    let mut state: AppState = expect_context();
    let del = db::delete_collection_cascade(&mut state.db, id)
        .await
        .srv()?;
    remove_poster(&state, del.poster_url.as_deref()).await;
    remove_media_files(&state, &del.media_paths).await;
    Ok(())
}

#[server]
pub async fn delete_item(id: Uuid) -> Result<(), ServerFnError> {
    use crate::app::server::{AppState, remove_media_files};
    let mut state: AppState = expect_context();
    let del = db::delete_item_cascade(&mut state.db, id).await.srv()?;
    remove_media_files(&state, &del.media_paths).await;
    Ok(())
}

#[server(input = server_fn::codec::MultipartFormData)]
pub async fn upload_collection_poster(
    data: server_fn::codec::MultipartData,
) -> Result<String, ServerFnError> {
    use crate::app::server::db::CollectionField;
    use crate::app::server::{AppState, poster::write_poster, upload::extension_of};

    let mut state: AppState = expect_context();

    let mut mp = data.into_inner().unwrap();
    let mut section_slug = String::new();
    let mut id: Uuid = Uuid::nil();
    let mut bytes: Option<(String, Vec<u8>)> = None;

    while let Some(f) = mp.next_field().await? {
        match f.name().unwrap_or("") {
            fields::SECTION => section_slug = f.text().await?,
            fields::ID => {
                id = Uuid::parse_str(&f.text().await?)
                    .map_err(|e| ServerFnError::new(format!("id: {e}")))?;
            }
            fields::POSTER_FILE => {
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

    let (ext, bytes) = bytes.ok_or_else(|| ServerFnError::new(messages::NO_IMAGE_RECEIVED))?;

    let collection = db::fetch_collection_detail(&mut state.db, &section_slug, id)
        .await
        .srv()?
        .ok_or_else(|| ServerFnError::new(messages::COLLECTION_NOT_FOUND))?;

    let url = write_poster(
        &state.config.storage.data_dir,
        &collection.section_slug,
        id,
        &ext,
        &bytes,
    )
    .await?;

    db::update_collection_field(&mut state.db, id, CollectionField::Poster, Some(&url))
        .await
        .srv()?;
    Ok(url)
}

#[server]
pub async fn move_item(id: Uuid, is_up: bool) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    db::swap_item_order(&mut state.db, id, is_up).await.srv()?;
    Ok(())
}

#[server]
pub async fn move_collection_to_section(
    collection_id: Uuid,
    to_section_slug: String,
) -> Result<(), ServerFnError> {
    use crate::app::server::db::{self, CollectionField, MoveCollectionError};
    use crate::app::server::{AppState, poster};

    let mut state: AppState = expect_context();

    let mv = db::move_collection_to_section(&mut state.db, collection_id, &to_section_slug)
        .await
        .map_err(|e| match e {
            MoveCollectionError::SectionNotFound => ServerFnError::new(messages::SECTION_NOT_FOUND),
            MoveCollectionError::MediaKindMismatch => {
                ServerFnError::new(messages::MEDIA_KIND_MISMATCH)
            }
            MoveCollectionError::Db(e) => {
                leptos::logging::error!("[db] {e}");
                ServerFnError::new(messages::INTERNAL_ERROR)
            }
        })?;

    if let Some(url) = mv.poster_url.as_deref()
        && let Some(new_url) = poster::move_poster_to_subdir(
            &state.config.storage.data_dir,
            &mv.from_slug,
            &to_section_slug,
            url,
        )
        .await
    {
        db::update_collection_field(
            &mut state.db,
            collection_id,
            CollectionField::Poster,
            Some(&new_url),
        )
        .await
        .srv()?;
    }

    Ok(())
}

#[server]
pub async fn fetch_collection_summaries(
    media_kind: String,
) -> Result<Vec<CollectionSummary>, ServerFnError> {
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    db::fetch_collection_summaries(&mut state.db, &media_kind)
        .await
        .srv()
}

#[server]
pub async fn move_items_to_collection(
    item_ids: Vec<Uuid>,
    to_collection_id: Uuid,
) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    db::move_items_to_collection(&mut state.db, &item_ids, to_collection_id)
        .await
        .srv()?;
    Ok(())
}
