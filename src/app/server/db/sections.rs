use sqlx::{SqliteConnection, SqlitePool};
use uuid::Uuid;

use super::tags::fetch_section_tags;
use crate::app::ids::Slug;
use crate::app::model::{AudioCollectionRow, MediaKind, Section, SectionRow, VideoCollectionRow};

// ─── Section queries ──────────────────────────────────────────────────────

pub async fn fetch_sections(pool: &SqlitePool) -> sqlx::Result<Vec<Section>> {
    let rows = sqlx::query_as!(
        SectionRow,
        r#"SELECT id            AS "id!: Uuid",
                  slug          AS "slug!: String",
                  title         AS "title!: String",
                  media_kind    AS "media_kind!: String",
                  nested        AS "nested!: bool",
                  position      AS "position!: i64"
           FROM sections
           ORDER BY position ASC, id ASC"#,
    )
    .fetch_all(pool)
    .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut s: Section = row.try_into()?;
        s.tags = fetch_section_tags(pool, s.id).await?;
        out.push(s);
    }
    Ok(out)
}

pub async fn fetch_section_by_slug(
    pool: &SqlitePool,
    slug: &Slug,
) -> sqlx::Result<Option<Section>> {
    let row = sqlx::query_as!(
        SectionRow,
        r#"SELECT id            AS "id!: Uuid",
                  slug          AS "slug!: String",
                  title         AS "title!: String",
                  media_kind    AS "media_kind!: String",
                  nested        AS "nested!: bool",
                  position      AS "position!: i64"
           FROM sections WHERE slug = ?"#,
        slug.as_str(),
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else { return Ok(None) };
    let mut s: Section = row.try_into()?;
    s.tags = fetch_section_tags(pool, s.id).await?;
    Ok(Some(s))
}

pub async fn insert_section(
    pool: &SqlitePool,
    slug: &Slug,
    title: &str,
    media_kind: MediaKind,
    nested: bool,
) -> sqlx::Result<Uuid> {
    let next: i64 = sqlx::query_scalar!(
        r#"SELECT COALESCE(MAX(position), -1) + 1 AS "n!: i64" FROM sections"#,
    )
    .fetch_one(pool)
    .await?;

    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO sections (id, slug, title, media_kind, nested, position)
           VALUES (?, ?, ?, ?, ?, ?)"#,
        id,
        slug.as_str(),
        title,
        media_kind.as_str(),
        nested,
        next,
    )
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn update_section(
    pool: &SqlitePool,
    id: Uuid,
    title: &str,
    media_kind: MediaKind,
    nested: bool,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"UPDATE sections SET title = ?, media_kind = ?, nested = ? WHERE id = ?"#,
        title,
        media_kind.as_str(),
        nested,
        id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_section_title(pool: &SqlitePool, id: Uuid, title: &str) -> sqlx::Result<()> {
    sqlx::query!(r#"UPDATE sections SET title = ? WHERE id = ?"#, title, id)
        .execute(pool)
        .await?;
    Ok(())
}

// ─── Section deletion ─────────────────────────────────────────────────────

pub enum DeleteSectionOutcome {
    Deleted,
    NotEmpty { collection_count: i64 },
}

pub async fn delete_section(pool: &SqlitePool, id: Uuid) -> sqlx::Result<DeleteSectionOutcome> {
    let slug: String = sqlx::query_scalar!(
        r#"SELECT slug AS "slug!: String" FROM sections WHERE id = ?"#,
        id,
    )
    .fetch_one(pool)
    .await?;

    let video_count: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "n!: i64" FROM video_collections WHERE section_slug = ?"#,
        slug,
    )
    .fetch_one(pool)
    .await?;

    let audio_count: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "n!: i64" FROM audio_collections WHERE section_slug = ?"#,
        slug,
    )
    .fetch_one(pool)
    .await?;

    let collection_count = video_count + audio_count;
    if collection_count > 0 {
        return Ok(DeleteSectionOutcome::NotEmpty { collection_count });
    }

    let mut tx = pool.begin().await?;
    sqlx::query!(r#"DELETE FROM section_tags WHERE section_id = ?"#, id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!(r#"DELETE FROM sections WHERE id = ?"#, id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(DeleteSectionOutcome::Deleted)
}

// ─── Upload-support ───────────────────────────────────────────────────────

pub async fn fetch_section_kind(pool: &SqlitePool, slug: &Slug) -> sqlx::Result<Option<MediaKind>> {
    let raw: Option<String> = sqlx::query_scalar!(
        r#"SELECT media_kind AS "media_kind!: String" FROM sections WHERE slug = ?"#,
        slug.as_str(),
    )
    .fetch_optional(pool)
    .await?;

    match raw {
        None => Ok(None),
        Some(s) => MediaKind::try_from(s.as_str())
            .map(Some)
            .map_err(|e| sqlx::Error::Protocol(format!("section has invalid media_kind: {e}"))),
    }
}

#[allow(dead_code)]
fn _unused(_: &VideoCollectionRow, _: &AudioCollectionRow, _: &mut SqliteConnection) {}
