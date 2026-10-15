use crate::app::error::AppError;
use crate::app::model::{Collection, CollectionSummary, Item};
use leptos::prelude::*;
use uuid::Uuid;

#[cfg(feature = "ssr")]
use crate::app::constants::messages;
#[cfg(feature = "ssr")]
use crate::app::error::ErrorContext;
#[cfg(feature = "ssr")]
use crate::app::server::db;
#[cfg(feature = "ssr")]
mod fields;

#[server]
pub async fn fetch_collections(
    section_slug: String,
    offset: usize,
    size: usize,
    search_query: Option<String>,
) -> Result<Vec<Collection>, AppError> {
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
    .context("fetch collections")
}

#[server]
pub async fn fetch_collections_count(
    section_slug: String,
    search_query: Option<String>,
) -> Result<usize, AppError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    let n = db::fetch_collections_count(&state.db, &section_slug, search_query.as_deref())
        .await
        .context("fetch collections count")?;
    Ok(n as usize)
}

#[server]
pub async fn fetch_collection_detail(
    section_slug: String,
    collection_id: Uuid,
) -> Result<Collection, AppError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    db::fetch_collection_detail(&state.db, &section_slug, collection_id)
        .await
        .context("fetch collection detail")?
        .ok_or_else(|| AppError::public(messages::COLLECTION_NOT_FOUND))
}

#[server]
pub async fn fetch_items(collection_id: Uuid) -> Result<Vec<Item>, AppError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    db::fetch_items(&state.db, collection_id)
        .await
        .context("fetch items")
}

#[server]
pub async fn patch_collection_field(
    id: Uuid,
    field: String,
    value: Option<String>,
) -> Result<(), AppError> {
    use crate::app::server::AppState;
    use crate::app::server::db::CollectionField;

    let state: AppState = expect_context();

    if field == "title" && value.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(AppError::validation(messages::TITLE_REQUIRED));
    }

    let f = match field.as_str() {
        "title" => CollectionField::Title,
        "description" => CollectionField::Description,
        _ => return Err(AppError::public(messages::UNKNOWN_FIELD)),
    };

    db::update_collection_field(&state.db, id, f, value.as_deref())
        .await
        .context("patch collection field")?;
    Ok(())
}

#[server]
pub async fn patch_collection_tags(id: Uuid, tags: Vec<String>) -> Result<(), AppError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    db::set_collection_tags(&state.db, id, &tags)
        .await
        .context("patch collection tags")?;
    Ok(())
}

#[server]
pub async fn patch_item_title(id: Uuid, title: String) -> Result<(), AppError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    let t = if title.trim().is_empty() {
        None
    } else {
        Some(title)
    };
    db::update_item_title(&state.db, id, t.as_deref())
        .await
        .context("patch item title")?;
    Ok(())
}

#[server]
pub async fn delete_collection(id: Uuid) -> Result<(), AppError> {
    use crate::app::server::{AppState, remove_media_files, remove_poster};
    let state: AppState = expect_context();
    let del = db::delete_collection_cascade(&state.db, id)
        .await
        .context("delete collection cascade")?;
    remove_media_files(&state, &del.media_paths).await;
    remove_poster(&state, del.poster_url.as_deref()).await;
    Ok(())
}

#[server]
pub async fn delete_item(id: Uuid) -> Result<(), AppError> {
    use crate::app::server::{AppState, remove_media_files, remove_poster};
    let state: AppState = expect_context();
    let del = db::delete_item_cascade(&state.db, id)
        .await
        .context("delete item cascade")?;
    remove_media_files(&state, &del.media_paths).await;
    remove_poster(&state, del.emptied_poster.as_deref()).await;
    Ok(())
}

#[server(input = server_fn::codec::MultipartFormData)]
pub async fn upload_collection_poster(
    data: server_fn::codec::MultipartData,
) -> Result<String, AppError> {
    use crate::app::model::MediaKind;
    use crate::app::server::AppState;
    use crate::app::server::poster::write_poster;
    use crate::app::server::upload::extension_of;

    let state: AppState = expect_context();

    let mut mp = data.into_inner().unwrap();
    let mut section_slug = String::new();
    let mut id: Uuid = Uuid::nil();
    let mut bytes: Option<(String, Vec<u8>)> = None;

    while let Some(f) = mp.next_field().await? {
        match f.name().unwrap_or("") {
            fields::SECTION => section_slug = f.text().await?,
            fields::ID => {
                id = Uuid::parse_str(&f.text().await?)
                    .map_err(|e| AppError::hidden("Invalid request", "poster id parse", e))?;
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

    let (ext, bytes) = bytes.ok_or_else(|| AppError::public(messages::NO_IMAGE_RECEIVED))?;

    let section = db::fetch_section_by_slug(&state.db, &section_slug)
        .await
        .context("fetch section for poster")?
        .ok_or_else(|| AppError::public(messages::SECTION_NOT_FOUND))?;
    if MediaKind::try_from(section.media_kind.as_str())
        .map_err(|e| AppError::internal("section has invalid media_kind", e))?
        != MediaKind::Audio
    {
        return Err(AppError::public(
            "This section's poster is generated automatically from its media",
        ));
    }

    let url = write_poster(
        &state.config.storage.data_dir,
        &section.slug,
        id,
        &ext,
        &bytes,
    )
    .await?;

    db::set_collection_poster(&state.db, id, Some(&url))
        .await
        .context("save regenerated poster")?;

    Ok(url)
}

#[server]
pub async fn regenerate_collection_poster(
    section_slug: String,
    collection_id: Uuid,
) -> Result<String, AppError> {
    use crate::app::model::MediaKind;
    use crate::app::server::AppState;
    use crate::app::server::convert::{ffprobe_duration, is_video_container_supported};
    use crate::app::server::poster::{extract_and_save_poster, pick_timestamp};

    let state: AppState = expect_context();

    let section = db::fetch_section_by_slug(&state.db, &section_slug)
        .await
        .context("fetch section for regenerate")?
        .ok_or_else(|| AppError::public(messages::SECTION_NOT_FOUND))?;
    if MediaKind::try_from(section.media_kind.as_str())
        .map_err(|e| AppError::internal("section has invalid media_kind", e))?
        != MediaKind::Video
    {
        return Err(AppError::public(
            "Only video sections have extractable posters",
        ));
    }

    let items = db::fetch_items(&state.db, collection_id)
        .await
        .context("fetch items for poster")?;
    let first = items
        .first()
        .ok_or_else(|| AppError::public("Collection has no items yet"))?;

    let rel = db::fetch_file_path(&state.db, first.file_id)
        .await
        .context("resolve first item file")?
        .ok_or_else(|| AppError::not_found("file"))?;

    let ext = rel.rsplit('.').next().unwrap_or("");
    if !is_video_container_supported(ext) {
        return Err(AppError::public("Cannot extract a poster from this file"));
    }

    let abs = state.config.storage.media_root.join(&rel);
    if tokio::fs::metadata(&abs).await.is_err() {
        return Err(AppError::public("Source file is missing"));
    }

    let previous = db::fetch_collection_detail(&state.db, &section_slug, collection_id)
        .await
        .context("fetch collection for poster regen")?
        .and_then(|c| c.poster);

    let duration = ffprobe_duration(&abs).await;
    let ts = pick_timestamp(duration, false);
    let url = extract_and_save_poster(
        &state.config.storage.data_dir,
        &section_slug,
        collection_id,
        &abs,
        ts,
    )
    .await?;

    if let Some(old) = previous.as_deref()
        && old != url
    {
        crate::app::server::remove_poster(&state, Some(old)).await;
    }

    db::set_collection_poster(&state.db, collection_id, Some(&url))
        .await
        .context("save regenerated poster")?;

    Ok(url)
}

#[server]
pub async fn move_item(id: Uuid, is_up: bool) -> Result<(), AppError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    db::swap_item_order(&state.db, id, is_up)
        .await
        .context("swap item order")?;
    Ok(())
}

#[server]
pub async fn move_collection_to_section(
    collection_id: Uuid,
    to_section_slug: String,
) -> Result<(), AppError> {
    use crate::app::server::db::{self as sdb, MoveCollectionError};
    use crate::app::server::{AppState, poster};

    let state: AppState = expect_context();

    let mv = sdb::move_collection_to_section(&state.db, collection_id, &to_section_slug)
        .await
        .map_err(|e| match e {
            MoveCollectionError::SectionNotFound => AppError::public(messages::SECTION_NOT_FOUND),
            MoveCollectionError::MediaKindMismatch => {
                AppError::public(messages::MEDIA_KIND_MISMATCH)
            }
            MoveCollectionError::Db(e) => AppError::internal("move collection to section", e),
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
        sdb::set_collection_poster(&state.db, collection_id, Some(&new_url))
            .await
            .context("save regenerated poster")?;
    }

    Ok(())
}

#[server]
pub async fn fetch_collection_summaries(
    media_kind: String,
) -> Result<Vec<CollectionSummary>, AppError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    db::fetch_collection_summaries(&state.db, &media_kind)
        .await
        .context("fetch collection summaries")
}

#[server]
pub async fn move_items_to_collection(
    item_ids: Vec<Uuid>,
    to_collection_id: Uuid,
) -> Result<(), AppError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    db::move_items_to_collection(&state.db, &item_ids, to_collection_id)
        .await
        .context("move items to collection")?;
    Ok(())
}
