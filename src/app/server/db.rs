//! Database layer. Public items are re-exported from this module, so
//! callers can keep using `server::db::X` without knowing the internal
//! split across files.

mod collections;
mod files;
mod items;
mod sections;
mod tags;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::str::FromStr;

use super::Config;

pub use collections::{
    CollectionDeletion, CollectionField, CollectionMove, MoveCollectionError,
    delete_collection_cascade, fetch_collection_any, fetch_collection_detail,
    fetch_collection_summaries, fetch_collections, fetch_collections_count,
    insert_audio_collection, insert_video_collection, move_collection_to_section,
    set_collection_poster, update_collection_field,
};
pub use files::{fetch_file_path, insert_file};

pub use items::{
    BulkItemDeletion, ItemDeletion, delete_item_cascade, delete_items_cascade, fetch_items,
    move_items_to_collection, swap_item_order, update_item_title,
};
pub use sections::{
    DeleteSectionOutcome, delete_section, fetch_section_by_slug, fetch_section_kind,
    fetch_sections, insert_section, update_section, update_section_title,
};
pub use tags::{
    fetch_all_tags, fetch_collection_tags, fetch_section_tags, set_collection_tags,
    set_section_tags,
};

// ─── Pool init ────────────────────────────────────────────────────────────────

pub async fn init(config: &Config) -> Result<sqlx::SqlitePool, Box<dyn std::error::Error>> {
    tokio::fs::create_dir_all(&config.storage.data_dir).await?;

    let db_path = config.db_path();
    let url = format!("sqlite://{}?mode=rwc", db_path.display());

    let opts = SqliteConnectOptions::from_str(&url)?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect_with(opts)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

// ─── Shared helpers ───────────────────────────────────────────────────────────

pub(crate) fn err_not_found(what: &str) -> sqlx::Error {
    let _ = what;
    sqlx::Error::RowNotFound
}
