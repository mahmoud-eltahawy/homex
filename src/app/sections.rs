use crate::app::model::Section;
use leptos::prelude::*;

#[cfg(feature = "ssr")]
use crate::app::server::{ToastyErr, db};

#[server]
pub async fn patch_section_title(id: u64, title: String) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    let t = title.trim();
    if t.is_empty() {
        return Err(ServerFnError::new("Title is required"));
    }
    db::update_section_title(&mut state.db, id as i64, t)
        .await
        .srv()?;
    Ok(())
}

#[server]
pub async fn fetch_sections() -> Result<Vec<Section>, ServerFnError> {
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    db::fetch_sections(&mut state.db).await.srv()
}

#[server]
pub async fn fetch_section_by_slug(slug: String) -> Result<Section, ServerFnError> {
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    db::fetch_section_by_slug(&mut state.db, &slug)
        .await
        .srv()?
        .ok_or_else(|| ServerFnError::new("section not found"))
}

#[server]
pub async fn create_section(
    title: String,
    media_kind: String,
    nested: bool,
) -> Result<u64, ServerFnError> {
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
        return Err(ServerFnError::new("Name is required"));
    }

    let kind = MediaKind::try_from(media_kind.as_str()).map_err(ServerFnError::new)?;
    let prefix = prefix_for(kind, nested);

    for attempt in 0..4 {
        let slug = format!("{prefix}-{}", short_hash());

        if db::fetch_section_by_slug(&mut state.db, &slug)
            .await
            .srv()?
            .is_some()
        {
            if attempt < 3 {
                continue;
            }
            return Err(ServerFnError::new(
                "Could not generate a unique section identifier",
            ));
        }

        match db::insert_section(&mut state.db, &slug, &title, media_kind.as_str(), nested).await {
            Ok(id) => return Ok(id),
            Err(e) => {
                leptos::logging::error!("[sections] insert failed: {e}");
                if attempt < 3 {
                    continue;
                }
                return Err(ServerFnError::new("Could not create the section"));
            }
        }
    }
    Err(ServerFnError::new(
        "Could not generate a unique section identifier",
    ))
}

#[server]
pub async fn update_section(
    id: u64,
    title: String,
    media_kind: String,
    nested: bool,
) -> Result<(), ServerFnError> {
    use crate::app::model::MediaKind;
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    MediaKind::try_from(media_kind.as_str()).map_err(ServerFnError::new)?;
    db::update_section(&mut state.db, id as i64, &title, &media_kind, nested)
        .await
        .srv()?;
    Ok(())
}

#[server]
pub async fn delete_section(id: u64) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    let mut state: AppState = expect_context();
    db::delete_section(&mut state.db, id as i64).await.srv()?;
    Ok(())
}
