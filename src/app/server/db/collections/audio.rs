use sqlx::SqlitePool;
use uuid::Uuid;

use super::super::err_not_found;
use super::super::files::purge_orphan_files;
use super::super::tags::{collection_ids_with_all_tags, fetch_collection_tags};
use super::{CollectionDeletion, CollectionMove, MoveCollectionError};
use crate::app::ids::Slug;
use crate::app::model::{
    AudioCollectionRow, Collection, CollectionSummary, MediaKind, SearchQuery, SectionRow,
    canonicalize_tags,
};
use crate::app::types::{PosterUrl, TagName, Title};

// ─── Read ────────────────────────────────────────────────────────────────────

pub(super) async fn fetch(
    pool: &SqlitePool,
    section_slug: &Slug,
    offset: usize,
    size: usize,
    search: Option<&SearchQuery>,
) -> sqlx::Result<Vec<Collection>> {
    let parsed = search.cloned().unwrap_or_default();

    let tag_ids: Option<Vec<Uuid>> = if parsed.tags.is_empty() {
        None
    } else {
        Some(collection_ids_with_all_tags(pool, &parsed.tags).await?)
    };
    if tag_ids.as_ref().is_some_and(Vec::is_empty) {
        return Ok(Vec::new());
    }

    let rows = sqlx::query_as!(
        AudioCollectionRow,
        r#"SELECT id AS "id!: Uuid", section_id AS "section_id!: Uuid",
                  section_slug, title, poster, description, position,
                  items_count AS "items_count!: i64"
           FROM audio_collections
           WHERE section_slug = ?1
             AND (?2 IS NULL OR title LIKE '%' || ?2 || '%')
           ORDER BY position ASC, title ASC, id ASC
           LIMIT ?3 OFFSET ?4"#,
        section_slug.as_str(),
        parsed.text,
        size as i64,
        offset as i64,
    )
    .fetch_all(pool)
    .await?;

    let rows: Vec<AudioCollectionRow> = if let Some(ids) = &tag_ids {
        rows.into_iter().filter(|r| ids.contains(&r.id)).collect()
    } else {
        rows
    };

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut c: Collection = row.into();
        c.tags = fetch_collection_tags(pool, c.id).await?;
        out.push(c);
    }
    Ok(out)
}

pub(super) async fn count(
    pool: &SqlitePool,
    section_slug: &Slug,
    search: Option<&SearchQuery>,
) -> sqlx::Result<i64> {
    let parsed = search.cloned().unwrap_or_default();

    let tag_ids: Option<Vec<Uuid>> = if parsed.tags.is_empty() {
        None
    } else {
        Some(collection_ids_with_all_tags(pool, &parsed.tags).await?)
    };
    if tag_ids.as_ref().is_some_and(Vec::is_empty) {
        return Ok(0);
    }

    if let Some(ids) = &tag_ids {
        let rows: Vec<Uuid> = sqlx::query_scalar!(
            r#"SELECT id AS "id!: Uuid"
               FROM audio_collections
               WHERE section_slug = ?1
                 AND (?2 IS NULL OR title LIKE '%' || ?2 || '%')"#,
            section_slug.as_str(),
            parsed.text,
        )
        .fetch_all(pool)
        .await?;
        return Ok(rows.into_iter().filter(|id| ids.contains(id)).count() as i64);
    }

    sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "n!: i64"
           FROM audio_collections
           WHERE section_slug = ?1
             AND (?2 IS NULL OR title LIKE '%' || ?2 || '%')"#,
        section_slug.as_str(),
        parsed.text,
    )
    .fetch_one(pool)
    .await
}

pub(super) async fn detail(
    pool: &SqlitePool,
    section_slug: &Slug,
    collection_id: Uuid,
) -> sqlx::Result<Option<Collection>> {
    let row = sqlx::query_as!(
        AudioCollectionRow,
        r#"SELECT id AS "id!: Uuid", section_id AS "section_id!: Uuid",
                  section_slug, title, poster, description, position,
                  items_count AS "items_count!: i64"
           FROM audio_collections
           WHERE section_slug = ? AND id = ?"#,
        section_slug.as_str(),
        collection_id,
    )
    .fetch_optional(pool)
    .await?;

    let Some(row) = row else { return Ok(None) };
    let mut c: Collection = row.into();
    c.tags = fetch_collection_tags(pool, c.id).await?;
    Ok(Some(c))
}

pub(super) async fn summaries(
    pool: &SqlitePool,
    media_kind: MediaKind,
) -> sqlx::Result<Vec<CollectionSummary>> {
    let sections = sqlx::query_as!(
        SectionRow,
        r#"SELECT id            AS "id!: Uuid",
                  slug          AS "slug!: String",
                  title         AS "title!: String",
                  media_kind    AS "media_kind!: String",
                  nested        AS "nested!: bool",
                  position      AS "position!: i64"
           FROM sections WHERE media_kind = ? ORDER BY title ASC"#,
        media_kind.as_str(),
    )
    .fetch_all(pool)
    .await?;

    let mut out: Vec<CollectionSummary> = Vec::new();
    for section in sections {
        let rows = sqlx::query_as!(
            AudioCollectionRow,
            r#"SELECT id AS "id!: Uuid", section_id AS "section_id!: Uuid",
                      section_slug, title, poster, description, position,
                      items_count AS "items_count!: i64"
               FROM audio_collections WHERE section_slug = ?
               ORDER BY title ASC"#,
            section.slug,
        )
        .fetch_all(pool)
        .await?;

        let section_title = Title::from_stored(section.title.clone());
        out.extend(rows.into_iter().map(|c| CollectionSummary {
            id: c.id,
            title: Title::from_stored(c.title),
            section_slug: Slug::from_stored(c.section_slug),
            section_title: section_title.clone(),
            nested: section.nested,
        }));
    }
    Ok(out)
}

// ─── Insert ──────────────────────────────────────────────────────────────────

async fn next_position(conn: &mut sqlx::SqliteConnection, section_id: Uuid) -> sqlx::Result<i64> {
    sqlx::query_scalar!(
        r#"SELECT COALESCE(MAX(position), -1) + 1 AS "n!: i64"
           FROM audio_collections WHERE section_id = ?"#,
        section_id,
    )
    .fetch_one(conn)
    .await
}

async fn write_tags(
    conn: &mut sqlx::SqliteConnection,
    collection_id: Uuid,
    tags: &[TagName],
) -> sqlx::Result<()> {
    for name in canonicalize_tags(tags.iter().cloned()) {
        let existing: Option<Uuid> = sqlx::query_scalar!(
            r#"SELECT id AS "id!: Uuid" FROM tags WHERE name = ?"#,
            name.as_str(),
        )
        .fetch_optional(&mut *conn)
        .await?;

        let tag_id = match existing {
            Some(id) => id,
            None => {
                let id = Uuid::new_v4();
                sqlx::query!(
                    r#"INSERT INTO tags (id, name) VALUES (?, ?)"#,
                    id,
                    name.as_str()
                )
                .execute(&mut *conn)
                .await?;
                id
            }
        };

        sqlx::query!(
            r#"INSERT INTO collection_tags (id, collection_id, tag_id) VALUES (?, ?, ?)"#,
            Uuid::new_v4(),
            collection_id,
            tag_id,
        )
        .execute(&mut *conn)
        .await?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn insert(
    pool: &SqlitePool,
    id: Uuid,
    section_slug: &Slug,
    title: &Title,
    description: Option<&str>,
    tags: &[TagName],
    poster: Option<PosterUrl>,
) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;

    let section = sqlx::query_as!(
        SectionRow,
        r#"SELECT id            AS "id!: Uuid",
                  slug          AS "slug!: String",
                  title         AS "title!: String",
                  media_kind    AS "media_kind!: String",
                  nested        AS "nested!: bool",
                  position      AS "position!: i64"
           FROM sections WHERE slug = ?"#,
        section_slug.as_str(),
    )
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| err_not_found("section"))?;

    let next = next_position(&mut tx, section.id).await?;
    let poster_str: Option<&str> = poster.as_ref().map(|p| p.as_str());

    sqlx::query!(
        r#"INSERT INTO audio_collections
           (id, section_id, section_slug, title, poster, description, position, items_count)
           VALUES (?, ?, ?, ?, ?, ?, ?, 0)"#,
        id,
        section.id,
        section.slug,
        title.as_str(),
        poster_str,
        description,
        next,
    )
    .execute(&mut *tx)
    .await?;

    write_tags(&mut tx, id, tags).await?;
    tx.commit().await?;
    Ok(())
}

// ─── Field / poster updates ──────────────────────────────────────────────────

pub(super) async fn update_description(
    pool: &SqlitePool,
    id: Uuid,
    value: Option<&str>,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"UPDATE audio_collections SET description = ? WHERE id = ?"#,
        value,
        id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub(super) async fn update_title(pool: &SqlitePool, id: Uuid, value: &Title) -> sqlx::Result<()> {
    sqlx::query!(
        r#"UPDATE audio_collections SET title = ? WHERE id = ?"#,
        value.as_str(),
        id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub(super) async fn set_poster(
    pool: &SqlitePool,
    id: Uuid,
    poster: &PosterUrl,
) -> sqlx::Result<()> {
    sqlx::query!(
        r#"UPDATE audio_collections SET poster = ? WHERE id = ?"#,
        poster.as_str(),
        id,
    )
    .execute(pool)
    .await?;
    Ok(())
}

// ─── Delete ──────────────────────────────────────────────────────────────────

pub(super) async fn delete_cascade(
    pool: &SqlitePool,
    id: Uuid,
) -> sqlx::Result<CollectionDeletion> {
    let mut tx = pool.begin().await?;

    let poster: Option<String> =
        sqlx::query_scalar!(r#"SELECT poster FROM audio_collections WHERE id = ?"#, id,)
            .fetch_one(&mut *tx)
            .await?;

    let file_ids: Vec<Uuid> = sqlx::query_scalar!(
        r#"SELECT file_id AS "file_id!: Uuid" FROM items WHERE collection_id = ?"#,
        id,
    )
    .fetch_all(&mut *tx)
    .await?;

    sqlx::query!(r#"DELETE FROM collection_tags WHERE collection_id = ?"#, id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!(r#"DELETE FROM audio_collections WHERE id = ?"#, id)
        .execute(&mut *tx)
        .await?;
    sqlx::query!(r#"DELETE FROM items WHERE collection_id = ?"#, id)
        .execute(&mut *tx)
        .await?;

    let media_paths = purge_orphan_files(&mut tx, file_ids).await?;

    tx.commit().await?;
    Ok(CollectionDeletion {
        media_paths,
        poster_url: poster.map(PosterUrl::from_stored),
    })
}

// ─── Move to another section ─────────────────────────────────────────────────

pub(super) async fn move_to_section(
    pool: &SqlitePool,
    collection_id: Uuid,
    to_section_slug: &Slug,
) -> Result<CollectionMove, MoveCollectionError> {
    let to_section = sqlx::query_as!(
        SectionRow,
        r#"SELECT id            AS "id!: Uuid",
                  slug          AS "slug!: String",
                  title         AS "title!: String",
                  media_kind    AS "media_kind!: String",
                  nested        AS "nested!: bool",
                  position      AS "position!: i64"
           FROM sections WHERE slug = ?"#,
        to_section_slug.as_str(),
    )
    .fetch_optional(pool)
    .await?
    .ok_or(MoveCollectionError::SectionNotFound)?;

    let to_kind = MediaKind::try_from(to_section.media_kind.as_str())
        .map_err(|_| MoveCollectionError::MediaKindMismatch)?;
    if to_kind != MediaKind::Audio {
        return Err(MoveCollectionError::MediaKindMismatch);
    }

    let (from_slug, poster): (String, Option<String>) = sqlx::query_as!(
        _FromSlug,
        r#"SELECT section_slug AS "section_slug!: String", poster
           FROM audio_collections WHERE id = ?"#,
        collection_id,
    )
    .fetch_one(pool)
    .await
    .map(|r| (r.section_slug, r.poster))?;

    sqlx::query!(
        r#"UPDATE audio_collections
           SET section_id = ?, section_slug = ?
           WHERE id = ?"#,
        to_section.id,
        to_section.slug,
        collection_id,
    )
    .execute(pool)
    .await?;

    Ok(CollectionMove {
        from_slug: Slug::from_stored(from_slug),
        poster_url: poster.map(PosterUrl::from_stored),
    })
}

#[derive(sqlx::FromRow)]
struct _FromSlug {
    section_slug: String,
    poster: Option<String>,
}
