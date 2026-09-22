use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Executor, Sqlite, SqlitePool};
use std::str::FromStr;

use super::Config;
use crate::app::model::{Collection, Item, MediaFile, MediaKind, Section};

// ─── Pool init ────────────────────────────────────────────────────────────

pub async fn init(config: &Config) -> Result<SqlitePool, Box<dyn std::error::Error>> {
    tokio::fs::create_dir_all(&config.storage.data_dir).await?;

    let opts = SqliteConnectOptions::from_str(&config.db_url())?
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
        .busy_timeout(std::time::Duration::from_secs(5))
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

// ─── Section queries ──────────────────────────────────────────────────────

pub async fn fetch_sections(pool: &SqlitePool) -> Result<Vec<Section>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT s.id, s.slug, s.title, s.media_kind, s.nested, s.position,
               COALESCE(
                   (SELECT COUNT(*) FROM collections c WHERE c.section_id = s.id),
                   0
               ) AS "collections_count!"
        FROM sections s
        ORDER BY s.position, s.id
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Section {
            id: r.id as u64,
            slug: r.slug,
            title: r.title,
            media_kind: MediaKind::try_from(r.media_kind.as_str()).unwrap_or(MediaKind::Video),
            nested: r.nested != 0,
            position: r.position,
            collections_count: r.collections_count as u32,
        })
        .collect())
}

pub async fn fetch_section_by_slug(
    pool: &SqlitePool,
    slug: &str,
) -> Result<Option<Section>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT s.id, s.slug, s.title, s.media_kind, s.nested, s.position,
               COALESCE(
                   (SELECT COUNT(*) FROM collections c WHERE c.section_id = s.id),
                   0
               ) AS "collections_count!"
        FROM sections s
        WHERE s.slug = ?
        "#,
        slug,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Section {
        id: r.id as u64,
        slug: r.slug,
        title: r.title,
        media_kind: MediaKind::try_from(r.media_kind.as_str()).unwrap_or(MediaKind::Video),
        nested: r.nested != 0,
        position: r.position,
        collections_count: r.collections_count as u32,
    }))
}

pub async fn insert_section<'e, E>(
    executor: E,
    slug: &str,
    title: &str,
    media_kind: &str,
    nested: bool,
) -> Result<i64, sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_scalar!(
        r#"
        INSERT INTO sections (slug, title, media_kind, nested, position)
        VALUES (?, ?, ?, ?,
                (SELECT COALESCE(MAX(position) + 1, 0) FROM sections))
        RETURNING id
        "#,
        slug,
        title,
        media_kind,
        nested as i64,
    )
    .fetch_one(executor)
    .await
}

pub async fn update_section<'e, E>(
    executor: E,
    id: i64,
    title: &str,
    media_kind: &str,
    nested: bool,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        "UPDATE sections SET title = ?, media_kind = ?, nested = ? WHERE id = ?",
        title,
        media_kind,
        nested as i64,
        id,
    )
    .execute(executor)
    .await
    .map(|_| ())
}

pub async fn delete_section<'e, E>(executor: E, id: i64) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!("DELETE FROM sections WHERE id = ?", id)
        .execute(executor)
        .await
        .map(|_| ())
}

// ─── Collection queries ───────────────────────────────────────────────────

pub async fn fetch_collections(
    pool: &SqlitePool,
    section_slug: &str,
    offset: usize,
    size: usize,
    search: Option<&str>,
) -> Result<Vec<Collection>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT c.id,
               c.section_id,
               s.slug AS section_slug,
               c.title,
               c.poster,
               c.description,
               COALESCE(
                   (SELECT COUNT(*) FROM items i WHERE i.collection_id = c.id),
                   0
               ) AS "items_count!"
        FROM collections c
        JOIN sections s ON s.id = c.section_id
        WHERE s.slug = ?1
          AND (?2 IS NULL OR c.title LIKE '%' || ?2 || '%')
        ORDER BY c.position, c.title
        LIMIT ?3 OFFSET ?4
        "#,
        section_slug,
        search,
        size as i64,
        offset as i64,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Collection {
            id: r.id as u64,
            section_id: r.section_id as u64,
            section_slug: r.section_slug,
            title: r.title,
            poster: r.poster,
            description: r.description,
            items_count: r.items_count as u32,
        })
        .collect())
}

pub async fn fetch_collections_count(
    pool: &SqlitePool,
    section_slug: &str,
    search: Option<&str>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)
        FROM collections c
        JOIN sections s ON s.id = c.section_id
        WHERE s.slug = ?1
          AND (?2 IS NULL OR c.title LIKE '%' || ?2 || '%')
        "#,
        section_slug,
        search,
    )
    .fetch_one(pool)
    .await
}

pub async fn fetch_collection_detail(
    pool: &SqlitePool,
    section_slug: &str,
    collection_id: i64,
) -> Result<Option<Collection>, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT c.id,
               c.section_id,
               s.slug AS section_slug,
               c.title,
               c.poster,
               c.description,
               COALESCE(
                   (SELECT COUNT(*) FROM items i WHERE i.collection_id = c.id),
                   0
               ) AS "items_count!"
        FROM collections c
        JOIN sections s ON s.id = c.section_id
        WHERE s.slug = ? AND c.id = ?
        "#,
        section_slug,
        collection_id,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Collection {
        id: r.id as u64,
        section_id: r.section_id as u64,
        section_slug: r.section_slug,
        title: r.title,
        poster: r.poster,
        description: r.description,
        items_count: r.items_count as u32,
    }))
}

pub async fn insert_empty_collection(
    pool: &SqlitePool,
    section_slug: &str,
) -> Result<i64, sqlx::Error> {
    let section_id: i64 =
        sqlx::query_scalar!("SELECT id FROM sections WHERE slug = ?", section_slug,)
            .fetch_one(pool)
            .await?;

    sqlx::query_scalar!(
        r#"
        INSERT INTO collections (section_id, title, position)
        VALUES (?1, 'بدون عنوان',
                (SELECT COALESCE(MAX(position) + 1, 0)
                 FROM collections WHERE section_id = ?1))
        RETURNING id AS "id!: i64"
        "#,
        section_id,
    )
    .fetch_one(pool)
    .await
}

pub async fn update_collection_title<'e, E>(
    executor: E,
    id: i64,
    value: Option<&str>,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!("UPDATE collections SET title = ? WHERE id = ?", value, id,)
        .execute(executor)
        .await
        .map(|_| ())
}

pub async fn update_collection_description<'e, E>(
    executor: E,
    id: i64,
    value: Option<&str>,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        "UPDATE collections SET description = ? WHERE id = ?",
        value,
        id,
    )
    .execute(executor)
    .await
    .map(|_| ())
}

pub async fn update_collection_poster<'e, E>(
    executor: E,
    id: i64,
    value: Option<&str>,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!("UPDATE collections SET poster = ? WHERE id = ?", value, id,)
        .execute(executor)
        .await
        .map(|_| ())
}

/// Dispatch wrapper for the string-driven `patch_collection_field` server fn.
/// Takes `&SqlitePool` (not a generic executor) because each branch produces
/// a different concrete query type.
pub async fn update_collection_by_field(
    pool: &SqlitePool,
    id: i64,
    field: &str,
    value: Option<&str>,
) -> Result<(), sqlx::Error> {
    match field {
        "title" => update_collection_title(pool, id, value).await,
        "description" => update_collection_description(pool, id, value).await,
        "poster" => update_collection_poster(pool, id, value).await,
        _ => Err(sqlx::Error::Protocol(format!("unknown field: {field}"))),
    }
}

// ─── Item queries ─────────────────────────────────────────────────────────

pub async fn fetch_items(pool: &SqlitePool, collection_id: i64) -> Result<Vec<Item>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT i.id,
               i.collection_id,
               i.number,
               i.season_number,
               i.title,
               i.poster,
               i.description,
               f.id            AS file_id,
               f.size_bytes    AS size,
               f.duration_secs AS dur
        FROM items i
        JOIN files f ON f.id = i.file_id
        WHERE i.collection_id = ?
        ORDER BY COALESCE(i.season_number, 0), i.number
        "#,
        collection_id,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Item {
            id: r.id as u64,
            collection_id: r.collection_id as u64,
            number: r.number,
            season_number: r.season_number,
            title: r.title,
            poster: r.poster,
            description: r.description,
            file: MediaFile {
                id: r.file_id as u64,
                path: format!("/media/{}", r.file_id),
                size: r.size as u64,
                duration: r.dur as u64,
            },
        })
        .collect())
}

pub async fn next_item_number<'e, E>(
    executor: E,
    collection_id: i64,
    season_number: Option<i64>,
) -> Result<i64, sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_scalar!(
        r#"
        SELECT COALESCE(MAX(number) + 1, 0)
        FROM items
        WHERE collection_id = ?1
          AND ((?2 IS NULL AND season_number IS NULL) OR season_number = ?2)
        "#,
        collection_id,
        season_number,
    )
    .fetch_one(executor)
    .await
}

pub async fn insert_item<'e, E>(
    executor: E,
    collection_id: i64,
    number: i64,
    season_number: Option<i64>,
    title: Option<&str>,
    file_id: i64,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!(
        r#"
        INSERT INTO items (collection_id, number, season_number, title, file_id)
        VALUES (?, ?, ?, ?, ?)
        "#,
        collection_id,
        number,
        season_number,
        title,
        file_id,
    )
    .execute(executor)
    .await
    .map(|_| ())
}

pub async fn update_item_title<'e, E>(
    executor: E,
    id: i64,
    title: Option<&str>,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!("UPDATE items SET title = ? WHERE id = ?", title, id)
        .execute(executor)
        .await
        .map(|_| ())
}

// ─── File queries ─────────────────────────────────────────────────────────

pub async fn insert_file<'e, E>(
    executor: E,
    relative_path: &str,
    size_bytes: i64,
    duration_secs: i64,
) -> Result<i64, sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_scalar!(
        r#"
        INSERT INTO files (relative_path, size_bytes, duration_secs)
        VALUES (?, ?, ?)
        RETURNING id
        "#,
        relative_path,
        size_bytes,
        duration_secs,
    )
    .fetch_one(executor)
    .await
}

pub async fn fetch_file_path(
    pool: &SqlitePool,
    file_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar!("SELECT relative_path FROM files WHERE id = ?", file_id,)
        .fetch_optional(pool)
        .await
}

// ─── Upload-support queries ───────────────────────────────────────────────

pub async fn fetch_section_kind(
    pool: &SqlitePool,
    slug: &str,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar!("SELECT media_kind FROM sections WHERE slug = ?", slug,)
        .fetch_optional(pool)
        .await
}

pub async fn update_section_title<'e, E>(
    executor: E,
    id: i64,
    title: &str,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query!("UPDATE sections SET title = ? WHERE id = ?", title, id)
        .execute(executor)
        .await
        .map(|_| ())
}

pub struct CollectionDeletion {
    pub media_paths: Vec<String>,
    pub poster_url: Option<String>,
}

pub struct ItemDeletion {
    pub media_paths: Vec<String>,
}

pub async fn delete_collection_cascade(
    pool: &SqlitePool,
    id: i64,
) -> Result<CollectionDeletion, sqlx::Error> {
    let mut tx = pool.begin().await?;

    // Grab the poster URL before the row disappears.
    let poster_url: Option<String> =
        sqlx::query_scalar!("SELECT poster FROM collections WHERE id = ?", id,)
            .fetch_optional(&mut *tx)
            .await?
            .flatten();

    // Snapshot the candidate file ids before the FK cascade removes items.
    let file_ids: Vec<i64> =
        sqlx::query_scalar!("SELECT file_id FROM items WHERE collection_id = ?", id,)
            .fetch_all(&mut *tx)
            .await?;

    sqlx::query!("DELETE FROM collections WHERE id = ?", id)
        .execute(&mut *tx)
        .await?;

    let media_paths = delete_orphaned_files(&mut tx, &file_ids).await?;

    tx.commit().await?;
    Ok(CollectionDeletion {
        media_paths,
        poster_url,
    })
}

pub async fn delete_item_cascade(pool: &SqlitePool, id: i64) -> Result<ItemDeletion, sqlx::Error> {
    let mut tx = pool.begin().await?;

    let file_id: Option<i64> = sqlx::query_scalar!("SELECT file_id FROM items WHERE id = ?", id,)
        .fetch_optional(&mut *tx)
        .await?;

    sqlx::query!("DELETE FROM items WHERE id = ?", id)
        .execute(&mut *tx)
        .await?;

    let file_ids: Vec<i64> = file_id.into_iter().collect();
    let media_paths = delete_orphaned_files(&mut tx, &file_ids).await?;

    tx.commit().await?;
    Ok(ItemDeletion { media_paths })
}

async fn delete_orphaned_files(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    file_ids: &[i64],
) -> Result<Vec<String>, sqlx::Error> {
    let mut removed = Vec::new();
    for fid in file_ids {
        let path: Option<String> = sqlx::query_scalar!(
            r#"
            DELETE FROM files
            WHERE id = ?
              AND NOT EXISTS (SELECT 1 FROM items WHERE file_id = ?)
            RETURNING relative_path
            "#,
            fid,
            fid,
        )
        .fetch_optional(&mut **tx)
        .await?;
        if let Some(p) = path {
            removed.push(p);
        }
    }
    Ok(removed)
}
