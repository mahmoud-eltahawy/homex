mod audio;
mod video;

use sqlx::SqlitePool;
use uuid::Uuid;

use super::err_not_found;
use crate::app::ids::Slug;
use crate::app::model::{Collection, CollectionKind, CollectionSummary, MediaKind, SearchQuery};
use crate::app::types::{PosterUrl, RelPath, TagName, Title};

// ─── Kind lookups ────────────────────────────────────────────────────────────

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

pub(crate) async fn section_kind(
    pool: &SqlitePool,
    section_slug: &Slug,
) -> sqlx::Result<MediaKind> {
    super::sections::fetch_section_kind(pool, section_slug)
        .await?
        .ok_or_else(|| err_not_found("section"))
}

// ─── Read entry points ───────────────────────────────────────────────────────

pub async fn fetch_collections(
    pool: &SqlitePool,
    section_slug: &Slug,
    offset: usize,
    size: usize,
    search: Option<&SearchQuery>,
) -> sqlx::Result<Vec<Collection>> {
    match section_kind(pool, section_slug).await? {
        MediaKind::Video => video::fetch(pool, section_slug, offset, size, search).await,
        MediaKind::Audio => audio::fetch(pool, section_slug, offset, size, search).await,
    }
}

pub async fn fetch_collections_count(
    pool: &SqlitePool,
    section_slug: &Slug,
    search: Option<&SearchQuery>,
) -> sqlx::Result<i64> {
    match section_kind(pool, section_slug).await? {
        MediaKind::Video => video::count(pool, section_slug, search).await,
        MediaKind::Audio => audio::count(pool, section_slug, search).await,
    }
}

pub async fn fetch_collection_detail(
    pool: &SqlitePool,
    section_slug: &Slug,
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
    media_kind: MediaKind,
) -> sqlx::Result<Vec<CollectionSummary>> {
    match media_kind {
        MediaKind::Video => video::summaries(pool, media_kind).await,
        MediaKind::Audio => audio::summaries(pool, media_kind).await,
    }
}

// ─── Insert entry points ─────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub async fn insert_video_collection(
    pool: &SqlitePool,
    id: Uuid,
    section_slug: &Slug,
    title: &Title,
    description: Option<&str>,
    tags: &[TagName],
    poster: &PosterUrl,
) -> sqlx::Result<()> {
    video::insert(pool, id, section_slug, title, description, tags, poster).await
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_audio_collection(
    pool: &SqlitePool,
    id: Uuid,
    section_slug: &Slug,
    title: &Title,
    description: Option<&str>,
    tags: &[TagName],
    poster: Option<PosterUrl>,
) -> sqlx::Result<()> {
    audio::insert(pool, id, section_slug, title, description, tags, poster).await
}

// ─── Field / poster updates ──────────────────────────────────────────────────

pub async fn update_collection_title(
    pool: &SqlitePool,
    id: Uuid,
    title: &Title,
) -> sqlx::Result<()> {
    let kind = kind_of(pool, id).await?;
    let Some(kind) = kind else {
        return Err(err_not_found("collection"));
    };
    match kind {
        CollectionKind::Video => video::update_title(pool, id, title).await,
        CollectionKind::Audio => audio::update_title(pool, id, title).await,
    }
}

pub async fn update_collection_description(
    pool: &SqlitePool,
    id: Uuid,
    value: &str,
) -> sqlx::Result<()> {
    let kind = kind_of(pool, id).await?;
    let Some(kind) = kind else {
        return Err(err_not_found("collection"));
    };
    match kind {
        CollectionKind::Video => video::update_description(pool, id, value).await,
        CollectionKind::Audio => audio::update_description(pool, id, value).await,
    }
}

pub async fn set_collection_poster(
    pool: &SqlitePool,
    id: Uuid,
    poster: &PosterUrl,
) -> sqlx::Result<()> {
    let kind = kind_of(pool, id).await?;
    let Some(kind) = kind else {
        return Err(err_not_found("collection"));
    };
    match kind {
        CollectionKind::Video => video::set_poster(pool, id, poster).await,
        CollectionKind::Audio => audio::set_poster(pool, id, poster).await,
    }
}

// ─── Delete / move ───────────────────────────────────────────────────────────

pub async fn delete_collection_cascade(
    pool: &SqlitePool,
    id: Uuid,
) -> sqlx::Result<CollectionDeletion> {
    let kind = kind_of(pool, id).await?;
    let Some(kind) = kind else {
        return Err(err_not_found("collection"));
    };
    match kind {
        CollectionKind::Video => video::delete_cascade(pool, id).await,
        CollectionKind::Audio => audio::delete_cascade(pool, id).await,
    }
}

pub async fn move_collection_to_section(
    pool: &SqlitePool,
    collection_id: Uuid,
    to_section_slug: &Slug,
) -> Result<CollectionMove, MoveCollectionError> {
    let kind = kind_of(pool, collection_id)
        .await
        .map_err(MoveCollectionError::Db)?;
    let Some(kind) = kind else {
        return Err(MoveCollectionError::Db(err_not_found("collection")));
    };
    match kind {
        CollectionKind::Video => video::move_to_section(pool, collection_id, to_section_slug).await,
        CollectionKind::Audio => audio::move_to_section(pool, collection_id, to_section_slug).await,
    }
}

// ─── Decrement-or-delete ─────────────────────────────────────────────────────

pub(crate) async fn decrement_or_delete(
    conn: &mut sqlx::SqliteConnection,
    collection_id: Uuid,
    delta: i64,
) -> sqlx::Result<Option<PosterUrl>> {
    let kind = kind_of_tx(conn, collection_id).await?;
    let Some(kind) = kind else {
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
                Ok(Some(PosterUrl::from_stored(poster)))
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
                Ok(poster.map(PosterUrl::from_stored))
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

// ─── Shared shapes ───────────────────────────────────────────────────────────

pub struct CollectionDeletion {
    pub media_paths: Vec<RelPath>,
    pub poster_url: Option<PosterUrl>,
}

pub struct CollectionMove {
    pub from_slug: Slug,
    pub poster_url: Option<PosterUrl>,
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
