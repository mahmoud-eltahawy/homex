use crate::app::error::AppError;
#[cfg(feature = "ssr")]
use crate::app::error::ErrorCode;
use crate::app::model::{MediaKind, Section};
use leptos::prelude::*;
use uuid::Uuid;

#[cfg(feature = "ssr")]
use crate::app::error::ErrorContext;
#[cfg(feature = "ssr")]
use crate::app::server::db;

#[server]
pub async fn patch_section_title(id: Uuid, title: String) -> Result<(), AppError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    let t = title.trim();
    if t.is_empty() {
        return Err(AppError::validation(ErrorCode::TitleRequired));
    }
    db::update_section_title(&state.db, id, t)
        .await
        .context("patch section title")?;
    Ok(())
}

#[server]
pub async fn patch_section_tags(id: Uuid, tags: Vec<String>) -> Result<(), AppError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    db::set_section_tags(&state.db, id, &tags)
        .await
        .context("patch section tags")?;
    Ok(())
}

#[server]
pub async fn fetch_sections() -> Result<Vec<Section>, AppError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    db::fetch_sections(&state.db)
        .await
        .context("fetch sections")
}

#[server]
pub async fn fetch_section_by_slug(slug: String) -> Result<Section, AppError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    db::fetch_section_by_slug(&state.db, &slug)
        .await
        .context("fetch section by slug")?
        .ok_or_else(|| AppError::public(ErrorCode::SectionNotFound))
}

#[cfg(feature = "ssr")]
pub fn prefix_for(kind: MediaKind, nested: bool) -> &'static str {
    match (kind, nested) {
        (MediaKind::Video, false) => "video",
        (MediaKind::Video, true) => "video-group",
        (MediaKind::Audio, false) => "audio",
        (MediaKind::Audio, true) => "audio-group",
    }
}

#[cfg(feature = "ssr")]
pub fn short_hash() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let ns = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let c = COUNTER.fetch_add(1, Ordering::Relaxed);
    let hash = (ns ^ c.wrapping_mul(0x9E37_79B9_7F4A_7C15)) as u32;
    format!("{hash:08x}")
}

#[server]
pub async fn create_section(
    title: String,
    media_kind: MediaKind,
    nested: bool,
) -> Result<Uuid, AppError> {
    use crate::app::server::AppState;

    let state: AppState = expect_context();

    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(AppError::validation(ErrorCode::NameRequired));
    }

    let prefix = prefix_for(media_kind, nested);

    for attempt in 0..4 {
        let slug = format!("{prefix}-{}", short_hash());

        if db::fetch_section_by_slug(&state.db, &slug)
            .await
            .context("check slug uniqueness")?
            .is_some()
        {
            if attempt < 3 {
                continue;
            }
            return Err(AppError::public(ErrorCode::UniqueSectionIdFailed));
        }

        match db::insert_section(&state.db, &slug, &title, media_kind.as_str(), nested).await {
            Ok(id) => return Ok(id),
            Err(e) => {
                if attempt < 3 {
                    leptos::logging::warn!("[sections] insert retry {attempt}: {e}");
                    continue;
                }
                return Err(AppError::internal("create section", e));
            }
        }
    }
    Err(AppError::public(ErrorCode::UniqueSectionIdFailed))
}

#[server]
pub async fn update_section(
    id: Uuid,
    title: String,
    media_kind: MediaKind,
    nested: bool,
) -> Result<(), AppError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    db::update_section(&state.db, id, &title, media_kind.as_str(), nested)
        .await
        .context("update section")?;
    Ok(())
}

#[server]
pub async fn delete_section(id: Uuid) -> Result<(), AppError> {
    use crate::app::server::AppState;
    use crate::app::server::db::DeleteSectionOutcome;

    let state: AppState = expect_context();
    match db::delete_section(&state.db, id)
        .await
        .context("delete section")?
    {
        DeleteSectionOutcome::Deleted => Ok(()),
        DeleteSectionOutcome::NotEmpty { collection_count } => {
            Err(AppError::public(ErrorCode::SectionNotEmpty {
                collection_count: collection_count as usize,
            }))
        }
    }
}
