use sqlx::AssertSqlSafe;
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

// ─── Row types (private) ──────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct SectionRow {
    id: i64,
    slug: String,
    title: String,
    media_kind: String,
    nested: i64,
    position: i64,
    collections_count: i64,
}

impl SectionRow {
    fn into_model(self) -> Section {
        Section {
            id: self.id as u64,
            slug: self.slug,
            title: self.title,
            media_kind: MediaKind::try_from(self.media_kind.as_str()).unwrap_or(MediaKind::Video),
            nested: self.nested != 0,
            position: self.position,
            collections_count: self.collections_count as u32,
        }
    }
}

#[derive(sqlx::FromRow)]
struct CollectionRow {
    id: i64,
    section_id: i64,
    section_slug: String,
    title: String,
    poster: Option<String>,
    description: Option<String>,
    items_count: i64,
}

impl CollectionRow {
    fn into_model(self) -> Collection {
        Collection {
            id: self.id as u64,
            section_id: self.section_id as u64,
            section_slug: self.section_slug,
            title: self.title,
            poster: self.poster,
            description: self.description,
            items_count: self.items_count as u32,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ItemRow {
    id: i64,
    collection_id: i64,
    number: i64,
    season_number: Option<i64>,
    title: Option<String>,
    poster: Option<String>,
    description: Option<String>,
    file_id: i64,
    size: i64,
    dur: i64,
}

impl ItemRow {
    fn into_model(self) -> Item {
        Item {
            id: self.id as u64,
            collection_id: self.collection_id as u64,
            number: self.number,
            season_number: self.season_number,
            title: self.title,
            poster: self.poster,
            description: self.description,
            file: MediaFile {
                id: self.file_id as u64,
                path: format!("/media/{}", self.file_id),
                size: self.size as u64,
                duration: self.dur as u64,
            },
        }
    }
}

// ─── Section queries ──────────────────────────────────────────────────────

const SECTION_SELECT: &str = "\
    SELECT s.id, s.slug, s.title, s.media_kind, s.nested, s.position, \
           (SELECT COUNT(*) FROM collections c WHERE c.section_id = s.id) AS collections_count \
    FROM sections s";

pub async fn fetch_sections(pool: &SqlitePool) -> Result<Vec<Section>, sqlx::Error> {
    let sql = AssertSqlSafe(format!("{SECTION_SELECT} ORDER BY s.position, s.id"));
    let rows: Vec<SectionRow> = sqlx::query_as(sql).fetch_all(pool).await?;
    Ok(rows.into_iter().map(SectionRow::into_model).collect())
}

pub async fn fetch_section_by_slug(
    pool: &SqlitePool,
    slug: &str,
) -> Result<Option<Section>, sqlx::Error> {
    let sql = AssertSqlSafe(format!("{SECTION_SELECT} WHERE s.slug = ?"));
    let row: Option<SectionRow> = sqlx::query_as(sql).bind(slug).fetch_optional(pool).await?;
    Ok(row.map(SectionRow::into_model))
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
    sqlx::query_scalar(
        "INSERT INTO sections (slug, title, media_kind, nested, position) \
         VALUES (?, ?, ?, ?, (SELECT COALESCE(MAX(position)+1, 0) FROM sections)) \
         RETURNING id",
    )
    .bind(slug)
    .bind(title)
    .bind(media_kind)
    .bind(nested as i64)
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
    sqlx::query("UPDATE sections SET title=?, media_kind=?, nested=? WHERE id=?")
        .bind(title)
        .bind(media_kind)
        .bind(nested as i64)
        .bind(id)
        .execute(executor)
        .await
        .map(|_| ())
}

pub async fn delete_section<'e, E>(executor: E, id: i64) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query("DELETE FROM sections WHERE id=?")
        .bind(id)
        .execute(executor)
        .await
        .map(|_| ())
}

// ─── Collection queries ───────────────────────────────────────────────────

const COLLECTION_SELECT: &str = "\
    SELECT c.id, c.section_id, s.slug AS section_slug, \
           c.title, c.poster, c.description, \
           (SELECT COUNT(*) FROM items i WHERE i.collection_id = c.id) AS items_count \
    FROM collections c JOIN sections s ON s.id = c.section_id";

pub async fn fetch_collections(
    pool: &SqlitePool,
    section_slug: &str,
    offset: usize,
    size: usize,
    search: Option<&str>,
) -> Result<Vec<Collection>, sqlx::Error> {
    let sql = AssertSqlSafe(format!(
        "{COLLECTION_SELECT} \
         WHERE s.slug = ?1 AND (?2 IS NULL OR c.title LIKE '%' || ?2 || '%') \
         ORDER BY c.position, c.title LIMIT ?3 OFFSET ?4"
    ));
    let rows: Vec<CollectionRow> = sqlx::query_as(sql)
        .bind(section_slug)
        .bind(search)
        .bind(size as i64)
        .bind(offset as i64)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(CollectionRow::into_model).collect())
}

pub async fn fetch_collections_count(
    pool: &SqlitePool,
    section_slug: &str,
    search: Option<&str>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM collections c JOIN sections s ON s.id = c.section_id \
         WHERE s.slug = ?1 AND (?2 IS NULL OR c.title LIKE '%' || ?2 || '%')",
    )
    .bind(section_slug)
    .bind(search)
    .fetch_one(pool)
    .await
}

pub async fn fetch_collection_detail(
    pool: &SqlitePool,
    section_slug: &str,
    collection_id: i64,
) -> Result<Option<Collection>, sqlx::Error> {
    let sql = AssertSqlSafe(format!("{COLLECTION_SELECT} WHERE s.slug = ? AND c.id = ?"));
    let row: Option<CollectionRow> = sqlx::query_as(sql)
        .bind(section_slug)
        .bind(collection_id)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(CollectionRow::into_model))
}

pub async fn insert_empty_collection(
    pool: &SqlitePool,
    section_slug: &str,
) -> Result<i64, sqlx::Error> {
    let section_id: i64 = sqlx::query_scalar("SELECT id FROM sections WHERE slug = ?")
        .bind(section_slug)
        .fetch_one(pool)
        .await?;

    sqlx::query_scalar(
        "INSERT INTO collections (section_id, title, position) \
         VALUES (?, 'بدون عنوان', \
                 (SELECT COALESCE(MAX(position)+1,0) FROM collections WHERE section_id=?)) \
         RETURNING id",
    )
    .bind(section_id)
    .bind(section_id)
    .fetch_one(pool)
    .await
}

/// Whitelisted mutable columns on `collections`. Passing this enum instead of
/// a raw `String` means no caller can smuggle a value into the SQL text.
#[derive(Clone, Copy)]
pub enum CollectionField {
    Title,
    Description,
    Poster,
}

impl CollectionField {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "title" => Some(Self::Title),
            "description" => Some(Self::Description),
            "poster" => Some(Self::Poster),
            _ => None,
        }
    }
    fn column(self) -> &'static str {
        match self {
            Self::Title => "title",
            Self::Description => "description",
            Self::Poster => "poster",
        }
    }
}

pub async fn update_collection_field<'e, E>(
    executor: E,
    id: i64,
    field: CollectionField,
    value: Option<&str>,
) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    let col = field.column();
    let sql = AssertSqlSafe(format!("UPDATE collections SET {col} = ? WHERE id = ?"));
    sqlx::query(sql)
        .bind(value)
        .bind(id)
        .execute(executor)
        .await
        .map(|_| ())
}

pub async fn delete_collection<'e, E>(executor: E, id: i64) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query("DELETE FROM collections WHERE id=?")
        .bind(id)
        .execute(executor)
        .await
        .map(|_| ())
}

// ─── Item queries ─────────────────────────────────────────────────────────

pub async fn fetch_items(pool: &SqlitePool, collection_id: i64) -> Result<Vec<Item>, sqlx::Error> {
    let rows: Vec<ItemRow> = sqlx::query_as(
        "SELECT i.id, i.collection_id, i.number, i.season_number, \
                i.title, i.poster, i.description, \
                f.id AS file_id, f.size_bytes AS size, f.duration_secs AS dur \
         FROM items i JOIN files f ON f.id = i.file_id \
         WHERE i.collection_id = ? \
         ORDER BY COALESCE(i.season_number, 0), i.number",
    )
    .bind(collection_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(ItemRow::into_model).collect())
}

pub async fn next_item_number<'e, E>(
    executor: E,
    collection_id: i64,
    season_number: Option<i64>,
) -> Result<i64, sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query_scalar(
        "SELECT COALESCE(MAX(number)+1, 0) FROM items \
         WHERE collection_id = ?1 AND \
         ((?2 IS NULL AND season_number IS NULL) OR season_number = ?2)",
    )
    .bind(collection_id)
    .bind(season_number)
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
    sqlx::query(
        "INSERT INTO items (collection_id, number, season_number, title, file_id) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(collection_id)
    .bind(number)
    .bind(season_number)
    .bind(title)
    .bind(file_id)
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
    sqlx::query("UPDATE items SET title = ? WHERE id = ?")
        .bind(title)
        .bind(id)
        .execute(executor)
        .await
        .map(|_| ())
}

pub async fn delete_item<'e, E>(executor: E, id: i64) -> Result<(), sqlx::Error>
where
    E: Executor<'e, Database = Sqlite>,
{
    sqlx::query("DELETE FROM items WHERE id=?")
        .bind(id)
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
    sqlx::query_scalar(
        "INSERT INTO files (relative_path, size_bytes, duration_secs) \
         VALUES (?, ?, ?) RETURNING id",
    )
    .bind(relative_path)
    .bind(size_bytes)
    .bind(duration_secs)
    .fetch_one(executor)
    .await
}

pub async fn fetch_file_path(
    pool: &SqlitePool,
    file_id: i64,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT relative_path FROM files WHERE id = ?")
        .bind(file_id)
        .fetch_optional(pool)
        .await
}

// ─── Upload-support queries ───────────────────────────────────────────────

pub async fn fetch_section_kind(
    pool: &SqlitePool,
    slug: &str,
) -> Result<Option<String>, sqlx::Error> {
    sqlx::query_scalar("SELECT media_kind FROM sections WHERE slug = ?")
        .bind(slug)
        .fetch_optional(pool)
        .await
}
