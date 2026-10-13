use crate::app::model::Section;
use leptos::prelude::*;
use uuid::Uuid;

#[cfg(feature = "ssr")]
use crate::app::constants::messages;

#[cfg(feature = "ssr")]
use crate::app::error::AppError;

#[cfg(feature = "ssr")]
use crate::app::server::{ToastyErr, db};

#[server]
pub async fn patch_section_title(id: Uuid, title: String) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    let t = title.trim();
    if t.is_empty() {
        return Err(AppError::validation(messages::TITLE_REQUIRED).into());
    }
    db::update_section_title(&mut state.db, id, t)
        .await
        .srv_ctx("patch section title")?;
    Ok(())
}

#[server]
pub async fn patch_section_tags(id: Uuid, tags: Vec<String>) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    db::set_section_tags(&mut state.db, id, &tags)
        .await
        .srv_ctx("patch section tags")?;
    Ok(())
}

#[server]
pub async fn fetch_sections() -> Result<Vec<Section>, ServerFnError> {
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    db::fetch_sections(&mut state.db)
        .await
        .srv_ctx("fetch sections")
}

#[server]
pub async fn fetch_section_by_slug(slug: String) -> Result<Section, ServerFnError> {
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    db::fetch_section_by_slug(&mut state.db, &slug)
        .await
        .srv_ctx("fetch section by slug")?
        .ok_or_else(|| AppError::not_found("section").into())
}

#[server]
pub async fn create_section(
    title: String,
    media_kind: String,
    nested: bool,
) -> Result<Uuid, ServerFnError> {
    use crate::app::model::MediaKind;
    use crate::app::server::AppState;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn prefix_for(kind: MediaKind, nested: bool) -> &'static str {
        match (kind, nested) {
            (MediaKind::Video, false) => "video",
            (MediaKind::Video, true) => "video-group",
            (MediaKind::Audio, false) => "audio",
            (MediaKind::Audio, true) => "audio-group",
        }
    }

    fn short_hash() -> String {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let ns = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);
        let c = COUNTER.fetch_add(1, Ordering::Relaxed);
        let hash = (ns ^ c.wrapping_mul(0x9E37_79B9_7F4A_7C15)) as u32;
        format!("{hash:08x}")
    }

    let mut state: AppState = expect_context();

    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(AppError::validation(messages::NAME_REQUIRED).into());
    }

    let kind = MediaKind::try_from(media_kind.as_str()).map_err(|e| AppError::public(e))?;
    let prefix = prefix_for(kind, nested);

    for attempt in 0..4 {
        let slug = format!("{prefix}-{}", short_hash());

        if db::fetch_section_by_slug(&mut state.db, &slug)
            .await
            .srv_ctx("check slug uniqueness")?
            .is_some()
        {
            if attempt < 3 {
                continue;
            }
            return Err(
                AppError::internal("create section", messages::UNIQUE_SECTION_ID_FAILED).into(),
            );
        }

        match db::insert_section(&mut state.db, &slug, &title, media_kind.as_str(), nested).await {
            Ok(id) => return Ok(id),
            Err(e) => {
                leptos::logging::error!("[sections] insert failed: {e}");
                if attempt < 3 {
                    continue;
                }
                return Err(AppError::internal("create section", e).into());
            }
        }
    }
    Err(AppError::internal("create section", messages::UNIQUE_SECTION_ID_FAILED).into())
}

#[server]
pub async fn update_section(
    id: Uuid,
    title: String,
    media_kind: String,
    nested: bool,
) -> Result<(), ServerFnError> {
    use crate::app::model::MediaKind;
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    MediaKind::try_from(media_kind.as_str()).map_err(AppError::public)?;
    db::update_section(&mut state.db, id, &title, &media_kind, nested)
        .await
        .srv_ctx("update section")?;
    Ok(())
}

#[server]
pub async fn delete_section(id: Uuid) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    use crate::app::server::db::DeleteSectionOutcome;

    let mut state: AppState = expect_context();
    match db::delete_section(&mut state.db, id)
        .await
        .srv_ctx("delete section")?
    {
        DeleteSectionOutcome::Deleted => Ok(()),
        DeleteSectionOutcome::NotEmpty { collection_count } => Err(AppError::public(format!(
            "{} ({} collection{})",
            messages::SECTION_NOT_EMPTY,
            collection_count,
            if collection_count == 1 { "" } else { "s" },
        ))
        .into()),
    }
}
