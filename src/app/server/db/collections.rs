mod audio;
mod video;

use sqlx::SqlitePool;
use uuid::Uuid;

use super::err_not_found;
use crate::app::model::{Collection, CollectionKind, CollectionSummary, MediaKind};

// ─── Kind lookups ─────────────────────────────────────────────────────────────

pub(crate) async fn kind_of(pool: &SqlitePool, id: Uuid) -> sqlx::Result<Option<CollectionKind>> {
    let video_hit: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "n!: i64" FROM video_collections WHERE id = ?"#,
        id,
    )
    .fetch_one(pool)
    .await?;
    if video_hit > 0 {
        return Ok(Some(CollectionKind::Video));
    }

    let audio_hit: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "n!: i64" FROM audio_collections WHERE id = ?"#,
        id,
    )
    .fetch_one(pool)
    .await?;
    if audio_hit > 0 {
        return Ok(Some(CollectionKind::Audio));
    }

    Ok(None)
}

pub(crate) async fn kind_of_tx(
    tx: &mut sqlx::SqliteConnection,
    id: Uuid,
) -> sqlx::Result<Option<CollectionKind>> {
    let video_hit: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "n!: i64" FROM video_collections WHERE id = ?"#,
        id,
    )
    .fetch_one(&mut *tx)
    .await?;
    if video_hit > 0 {
        return Ok(Some(CollectionKind::Video));
    }

    let audio_hit: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "n!: i64" FROM audio_collections WHERE id = ?"#,
        id,
    )
    .fetch_one(&mut *tx)
    .await?;
    if audio_hit > 0 {
        return Ok(Some(CollectionKind::Audio));
    }

    Ok(None)
}

pub(crate) async fn section_kind(pool: &SqlitePool, section_slug: &str) -> sqlx::Result<MediaKind> {
    let mk: Option<String> = sqlx::query_scalar!(
        r#"SELECT media_kind AS "media_kind!: String" FROM sections WHERE slug = ?"#,
        section_slug,
    )
    .fetch_optional(pool)
    .await?;
    let mk = mk.ok_or_else(|| err_not_found("section"))?;
    MediaKind::try_from(mk.as_str())
        .map_err(|e| sqlx::Error::Protocol(format!("section has invalid media_kind: {e}")))
}

// ─── Read entry points ────────────────────────────────────────────────────────

pub async fn fetch_collections(
    pool: &SqlitePool,
    section_slug: &str,
    offset: usize,
    size: usize,
    search: Option<&str>,
) -> sqlx::Result<Vec<Collection>> {
    match section_kind(pool, section_slug).await? {
        MediaKind::Video => video::fetch(pool, section_slug, offset, size, search).await,
        MediaKind::Audio => audio::fetch(pool, section_slug, offset, size, search).await,
    }
}

pub async fn fetch_collections_count(
    pool: &SqlitePool,
    section_slug: &str,
    search: Option<&str>,
) -> sqlx::Result<i64> {
    match section_kind(pool, section_slug).await? {
        MediaKind::Video => video::count(pool, section_slug, search).await,
        MediaKind::Audio => audio::count(pool, section_slug, search).await,
    }
}

pub async fn fetch_collection_detail(
    pool: &SqlitePool,
    section_slug: &str,
    collection_id: Uuid,
) -> sqlx::Result<Option<Collection>> {
    match section_kind(pool, section_slug).await? {
        MediaKind::Video => video::detail(pool, section_slug, collection_id).await,
        MediaKind::Audio => audio::detail(pool, section_slug, collection_id).await,
    }
}

pub async fn fetch_collection_any(pool: &SqlitePool, id: Uuid) -> sqlx::Result<Option<Collection>> {
    use super::tags::fetch_collection_tags;

    match kind_of(pool, id).await? {
        None => Ok(None),
        Some(CollectionKind::Video) => {
            let row = sqlx::query_as!(
                crate::app::model::VideoCollectionRow,
                r#"SELECT id AS "id!: Uuid", section_id AS "section_id!: Uuid",
                          section_slug, title, poster, description, position,
                          items_count AS "items_count!: i64"
                   FROM video_collections WHERE id = ?"#,
                id,
            )
            .fetch_optional(pool)
            .await?;
            let Some(row) = row else { return Ok(None) };
            let mut c: Collection = row.into();
            c.tags = fetch_collection_tags(pool, c.id).await?;
            Ok(Some(c))
        }
        Some(CollectionKind::Audio) => {
            let row = sqlx::query_as!(
                crate::app::model::AudioCollectionRow,
                r#"SELECT id AS "id!: Uuid", section_id AS "section_id!: Uuid",
                          section_slug, title, poster, description, position,
                          items_count AS "items_count!: i64"
                   FROM audio_collections WHERE id = ?"#,
                id,
            )
            .fetch_optional(pool)
            .await?;
            let Some(row) = row else { return Ok(None) };
            let mut c: Collection = row.into();
            c.tags = fetch_collection_tags(pool, c.id).await?;
            Ok(Some(c))
        }
    }
}

pub async fn fetch_collection_summaries(
    pool: &SqlitePool,
    media_kind: &str,
) -> sqlx::Result<Vec<CollectionSummary>> {
    match MediaKind::try_from(media_kind)
        .map_err(|e| sqlx::Error::Protocol(format!("bad media_kind: {e}")))?
    {
        MediaKind::Video => video::summaries(pool, media_kind).await,
        MediaKind::Audio => audio::summaries(pool, media_kind).await,
    }
}

// ─── Insert entry points ──────────────────────────────────────────────────────

pub async fn insert_video_collection(
    pool: &SqlitePool,
    id: Uuid,
    section_slug: &str,
    title: &str,
    description: Option<&str>,
    tags: &[String],
    poster: &str,
) -> sqlx::Result<()> {
    video::insert(pool, id, section_slug, title, description, tags, poster).await
}

pub async fn insert_audio_collection(
    pool: &SqlitePool,
    id: Uuid,
    section_slug: &str,
    title: &str,
    description: Option<&str>,
    tags: &[String],
    poster: Option<String>,
) -> sqlx::Result<()> {
    audio::insert(pool, id, section_slug, title, description, tags, poster).await
}

// ─── Field / poster updates ───────────────────────────────────────────────────

pub async fn update_collection_field(
    pool: &SqlitePool,
    id: Uuid,
    field: CollectionField,
    value: Option<&str>,
) -> sqlx::Result<()> {
    match kind_of(pool, id)
        .await?
        .ok_or_else(|| err_not_found("collection"))?
    {
        CollectionKind::Video => video::update_field(pool, id, field, value).await,
        CollectionKind::Audio => audio::update_field(pool, id, field, value).await,
    }
}

pub async fn set_collection_poster(
    pool: &SqlitePool,
    id: Uuid,
    url: Option<&str>,
) -> sqlx::Result<()> {
    match kind_of(pool, id)
        .await?
        .ok_or_else(|| err_not_found("collection"))?
    {
        CollectionKind::Video => {
            let url = url.ok_or_else(|| {
                sqlx::Error::Protocol("video collection poster cannot be cleared".into())
            })?;
            video::set_poster(pool, id, url).await
        }
        CollectionKind::Audio => audio::set_poster(pool, id, url).await,
    }
}

// ─── Delete / move ────────────────────────────────────────────────────────────

pub async fn delete_collection_cascade(
    pool: &SqlitePool,
    id: Uuid,
) -> sqlx::Result<CollectionDeletion> {
    match kind_of(pool, id)
        .await?
        .ok_or_else(|| err_not_found("collection"))?
    {
        CollectionKind::Video => video::delete_cascade(pool, id).await,
        CollectionKind::Audio => audio::delete_cascade(pool, id).await,
    }
}

pub async fn move_collection_to_section(
    pool: &SqlitePool,
    collection_id: Uuid,
    to_section_slug: &str,
) -> Result<CollectionMove, MoveCollectionError> {
    match kind_of(pool, collection_id)
        .await
        .map_err(MoveCollectionError::Db)?
        .ok_or_else(|| MoveCollectionError::Db(err_not_found("collection")))?
    {
        CollectionKind::Video => video::move_to_section(pool, collection_id, to_section_slug).await,
        CollectionKind::Audio => audio::move_to_section(pool, collection_id, to_section_slug).await,
    }
}

// ─── Decrement-or-delete helper ───────────────────────────────────────────────

pub(crate) async fn decrement_or_delete(
    conn: &mut sqlx::SqliteConnection,
    collection_id: Uuid,
    delta: i64,
) -> sqlx::Result<Option<String>> {
    let Some(kind) = kind_of_tx(conn, collection_id).await? else {
        return Ok(None);
    };

    match kind {
        CollectionKind::Video => {
            let (count, poster): (i64, String) = sqlx::query_as!(
                _VideoCount,
                r#"SELECT items_count AS "items_count!: i64", poster AS "poster!: String"
                   FROM video_collections WHERE id = ?"#,
                collection_id,
            )
            .fetch_one(&mut *conn)
            .await
            .map(|r| (r.items_count, r.poster))?;

            let next = (count - delta).max(0);
            if next == 0 {
                sqlx::query!(
                    r#"DELETE FROM collection_tags WHERE collection_id = ?"#,
                    collection_id
                )
                .execute(&mut *conn)
                .await?;
                sqlx::query!(
                    r#"DELETE FROM video_collections WHERE id = ?"#,
                    collection_id
                )
                .execute(&mut *conn)
                .await?;
                Ok(Some(poster))
            } else {
                sqlx::query!(
                    r#"UPDATE video_collections SET items_count = ? WHERE id = ?"#,
                    next,
                    collection_id,
                )
                .execute(&mut *conn)
                .await?;
                Ok(None)
            }
        }
        CollectionKind::Audio => {
            let (count, poster): (i64, Option<String>) = sqlx::query_as!(
                _AudioCount,
                r#"SELECT items_count AS "items_count!: i64", poster
                   FROM audio_collections WHERE id = ?"#,
                collection_id,
            )
            .fetch_one(&mut *conn)
            .await
            .map(|r| (r.items_count, r.poster))?;

            let next = (count - delta).max(0);
            if next == 0 {
                sqlx::query!(
                    r#"DELETE FROM collection_tags WHERE collection_id = ?"#,
                    collection_id
                )
                .execute(&mut *conn)
                .await?;
                sqlx::query!(
                    r#"DELETE FROM audio_collections WHERE id = ?"#,
                    collection_id
                )
                .execute(&mut *conn)
                .await?;
                Ok(poster)
            } else {
                sqlx::query!(
                    r#"UPDATE audio_collections SET items_count = ? WHERE id = ?"#,
                    next,
                    collection_id,
                )
                .execute(&mut *conn)
                .await?;
                Ok(None)
            }
        }
    }
}

#[derive(sqlx::FromRow)]
struct _VideoCount {
    items_count: i64,
    poster: String,
}

#[derive(sqlx::FromRow)]
struct _AudioCount {
    items_count: i64,
    poster: Option<String>,
}

// ─── Shared shapes ────────────────────────────────────────────────────────────

pub struct CollectionDeletion {
    pub media_paths: Vec<String>,
    pub poster_url: Option<String>,
}

pub struct CollectionMove {
    pub from_slug: String,
    pub poster_url: Option<String>,
}

#[derive(Debug)]
pub enum MoveCollectionError {
    SectionNotFound,
    MediaKindMismatch,
    Db(sqlx::Error),
}

impl From<sqlx::Error> for MoveCollectionError {
    fn from(e: sqlx::Error) -> Self {
        MoveCollectionError::Db(e)
    }
}

pub enum CollectionField {
    Title,
    Description,
    Poster,
}
