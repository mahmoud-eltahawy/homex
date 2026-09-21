use crate::app::model::Section;
use leptos::prelude::*;

#[server]
pub async fn fetch_sections() -> Result<Vec<Section>, ServerFnError> {
    use crate::app::model::MediaKind;
    use crate::app::server::AppState;

    let state: AppState = expect_context();

    #[derive(sqlx::FromRow)]
    struct Row {
        id: i64,
        slug: String,
        title: String,
        media_kind: String,
        nested: i64,
        position: i64,
        collections_count: i64,
    }

    let rows: Vec<Row> = sqlx::query_as(
    "SELECT s.id, s.slug, s.title, s.media_kind, s.nested, s.position, \
                (SELECT COUNT(*) FROM collections c WHERE c.section_id = s.id) AS collections_count \
         FROM sections s ORDER BY s.position, s.id",
)
.fetch_all(&state.db).await
.map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|r| Section {
            id: r.id as u64,
            slug: r.slug,
            title: r.title,
            media_kind: MediaKind::try_from(r.media_kind.as_str()).unwrap(),
            nested: r.nested != 0,
            position: r.position,
            collections_count: r.collections_count as u32,
        })
        .collect())
}

#[server]
pub async fn fetch_section_by_slug(slug: String) -> Result<Section, ServerFnError> {
    use crate::app::model::MediaKind;
    use crate::app::server::AppState;
    let state: AppState = expect_context();

    #[derive(sqlx::FromRow)]
    struct Row {
        id: i64,
        slug: String,
        title: String,
        media_kind: String,
        nested: i64,
        position: i64,
        collections_count: i64,
    }
    let r: Row = sqlx::query_as(
    "SELECT s.id, s.slug, s.title, s.media_kind, s.nested, s.position, \
                (SELECT COUNT(*) FROM collections c WHERE c.section_id = s.id) AS collections_count \
         FROM sections s WHERE s.slug = ?",
)
.bind(&slug)
.fetch_optional(&state.db).await
.map_err(|e| ServerFnError::new(e.to_string()))?
.ok_or_else(|| ServerFnError::new("section not found"))?;

    Ok(Section {
        id: r.id as u64,
        slug: r.slug,
        title: r.title,
        media_kind: MediaKind::try_from(r.media_kind.as_str()).unwrap(),
        nested: r.nested != 0,
        position: r.position,
        collections_count: r.collections_count as u32,
    })
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

    /// Section-type → URL prefix. This is the *only* part of the slug we
    /// derive from user input, and it comes from the enum, not the title.
    fn prefix_for(kind: MediaKind, nested: bool) -> &'static str {
        match (kind, nested) {
            (MediaKind::Video, false) => "video",
            (MediaKind::Video, true) => "video-group",
            (MediaKind::Audio, false) => "audio",
            (MediaKind::Audio, true) => "audio-group",
        }
    }

    /// 8 hex chars derived from wall-clock nanoseconds XOR a process-local
    /// counter. Collision probability per request is negligible; the INSERT
    /// loop below handles the astronomically unlikely case anyway.
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

    let state: AppState = expect_context();

    let title = title.trim().to_string();
    if title.is_empty() {
        return Err(ServerFnError::new("الاسم مطلوب"));
    }

    let kind = MediaKind::try_from(media_kind.as_str()).map_err(ServerFnError::new)?;
    let prefix = prefix_for(kind, nested);

    // Retry on the (practically impossible) unique-constraint collision.
    for attempt in 0..4 {
        let slug = format!("{prefix}-{}", short_hash());
        let res: Result<u64, sqlx::Error> = sqlx::query_scalar(
            "INSERT INTO sections (slug, title, media_kind, nested, position) \
             VALUES (?, ?, ?, ?, (SELECT COALESCE(MAX(position)+1, 0) FROM sections)) \
             RETURNING id",
        )
        .bind(&slug)
        .bind(&title)
        .bind(media_kind.as_str())
        .bind(nested as i64)
        .fetch_one(&state.db)
        .await;

        match res {
            Ok(id) => return Ok(id as u64),
            Err(sqlx::Error::Database(e)) if e.is_unique_violation() && attempt < 3 => {
                continue;
            }
            Err(e) => return Err(ServerFnError::new(e.to_string())),
        }
    }
    Err(ServerFnError::new("تعذّر توليد معرّف فريد للقسم"))
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
    let state: AppState = expect_context();
    MediaKind::try_from(media_kind.as_str()).map_err(ServerFnError::new)?;

    sqlx::query("UPDATE sections SET title=?, media_kind=?, nested=? WHERE id=?")
        .bind(&title)
        .bind(&media_kind)
        .bind(nested as i64)
        .bind(id as i64)
        .execute(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[server]
pub async fn delete_section(id: u64) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    sqlx::query("DELETE FROM sections WHERE id=?")
        .bind(id as i64)
        .execute(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}
