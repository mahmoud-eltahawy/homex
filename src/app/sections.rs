use crate::app::model::{MediaKind, Section};
use leptos::prelude::*;

#[server]
pub async fn fetch_sections() -> Result<Vec<Section>, ServerFnError> {
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
    slug: String,
    title: String,
    media_kind: String,
    nested: bool,
) -> Result<u64, ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();

    let slug = slug.trim().to_ascii_lowercase();
    if slug.is_empty()
        || !slug
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(ServerFnError::new("المعرّف (slug) غير صالح"));
    }
    MediaKind::try_from(media_kind.as_str()).map_err(ServerFnError::new)?;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO sections (slug, title, media_kind, nested, position) \
         VALUES (?, ?, ?, ?, (SELECT COALESCE(MAX(position)+1, 0) FROM sections)) \
         RETURNING id",
    )
    .bind(&slug)
    .bind(&title)
    .bind(&media_kind)
    .bind(nested as i64)
    .fetch_one(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(id as u64)
}

#[server]
pub async fn update_section(
    id: u64,
    title: String,
    media_kind: String,
    nested: bool,
) -> Result<(), ServerFnError> {
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
