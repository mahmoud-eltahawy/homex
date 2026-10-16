use std::collections::HashMap;

use sqlx::{SqliteConnection, SqlitePool};
use uuid::Uuid;

use super::collections::decrement_or_delete;
use super::files::purge_orphan_files;
use crate::app::model::Item;
use crate::app::types::{PosterUrl, RelPath};

// ─── Item queries ─────────────────────────────────────────────────────────

pub async fn fetch_items(pool: &SqlitePool, collection_id: Uuid) -> sqlx::Result<Vec<Item>> {
    sqlx::query_as!(
        Item,
        r#"SELECT id            AS "id!: Uuid",
                  collection_id AS "collection_id!: Uuid",
                  number        AS "number!: i64",
                  season_number,
                  title,
                  poster,
                  description,
                  file_id       AS "file_id!: Uuid"
           FROM items
           WHERE collection_id = ?
           ORDER BY season_number ASC, number ASC, id ASC"#,
        collection_id,
    )
    .fetch_all(pool)
    .await
}

pub async fn update_item_title(
    pool: &SqlitePool,
    id: Uuid,
    title: Option<&str>,
) -> sqlx::Result<()> {
    sqlx::query!(r#"UPDATE items SET title = ? WHERE id = ?"#, title, id)
        .execute(pool)
        .await?;
    Ok(())
}

// ─── Reordering ───────────────────────────────────────────────────────────

pub async fn swap_item_order(pool: &SqlitePool, id: Uuid, is_up: bool) -> sqlx::Result<()> {
    let mut tx = pool.begin().await?;

    let item = sqlx::query_as!(
        Item,
        r#"SELECT id            AS "id!: Uuid",
                  collection_id AS "collection_id!: Uuid",
                  number        AS "number!: i64",
                  season_number, title, poster, description,
                  file_id       AS "file_id!: Uuid"
           FROM items WHERE id = ?"#,
        id,
    )
    .fetch_one(&mut *tx)
    .await?;

    let neighbour = match (is_up, item.season_number) {
        (true, Some(s)) => {
            sqlx::query_as!(
                Item,
                r#"SELECT id AS "id!: Uuid", collection_id AS "collection_id!: Uuid",
                      number AS "number!: i64", season_number, title, poster,
                      description, file_id AS "file_id!: Uuid"
               FROM items
               WHERE collection_id = ? AND season_number = ? AND number < ?
               ORDER BY number DESC LIMIT 1"#,
                item.collection_id,
                s,
                item.number,
            )
            .fetch_optional(&mut *tx)
            .await?
        }
        (false, Some(s)) => {
            sqlx::query_as!(
                Item,
                r#"SELECT id AS "id!: Uuid", collection_id AS "collection_id!: Uuid",
                      number AS "number!: i64", season_number, title, poster,
                      description, file_id AS "file_id!: Uuid"
               FROM items
               WHERE collection_id = ? AND season_number = ? AND number > ?
               ORDER BY number ASC LIMIT 1"#,
                item.collection_id,
                s,
                item.number,
            )
            .fetch_optional(&mut *tx)
            .await?
        }
        (true, None) => {
            sqlx::query_as!(
                Item,
                r#"SELECT id AS "id!: Uuid", collection_id AS "collection_id!: Uuid",
                      number AS "number!: i64", season_number, title, poster,
                      description, file_id AS "file_id!: Uuid"
               FROM items
               WHERE collection_id = ? AND season_number IS NULL AND number < ?
               ORDER BY number DESC LIMIT 1"#,
                item.collection_id,
                item.number,
            )
            .fetch_optional(&mut *tx)
            .await?
        }
        (false, None) => {
            sqlx::query_as!(
                Item,
                r#"SELECT id AS "id!: Uuid", collection_id AS "collection_id!: Uuid",
                      number AS "number!: i64", season_number, title, poster,
                      description, file_id AS "file_id!: Uuid"
               FROM items
               WHERE collection_id = ? AND season_number IS NULL AND number > ?
               ORDER BY number ASC LIMIT 1"#,
                item.collection_id,
                item.number,
            )
            .fetch_optional(&mut *tx)
            .await?
        }
    };

    let Some(neighbour) = neighbour else {
        tx.commit().await?;
        return Ok(());
    };

    let (a_number, b_number) = (item.number, neighbour.number);
    sqlx::query!(
        r#"UPDATE items SET number = ? WHERE id = ?"#,
        b_number,
        item.id
    )
    .execute(&mut *tx)
    .await?;
    sqlx::query!(
        r#"UPDATE items SET number = ? WHERE id = ?"#,
        a_number,
        neighbour.id
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

// ─── Move items between collections ───────────────────────────────────────

pub async fn move_items_to_collection(
    pool: &SqlitePool,
    item_ids: &[Uuid],
    to_collection_id: Uuid,
) -> sqlx::Result<()> {
    use super::collections::kind_of;
    use crate::app::model::CollectionKind;

    if item_ids.is_empty() {
        return Ok(());
    }

    let dest_kind: CollectionKind = kind_of(pool, to_collection_id)
        .await?
        .ok_or_else(|| super::err_not_found("collection"))?;

    let mut tx = pool.begin().await?;

    match dest_kind {
        CollectionKind::Video => {
            sqlx::query!(
                r#"SELECT id FROM video_collections WHERE id = ?"#,
                to_collection_id
            )
            .fetch_one(&mut *tx)
            .await?;
        }
        CollectionKind::Audio => {
            sqlx::query!(
                r#"SELECT id FROM audio_collections WHERE id = ?"#,
                to_collection_id
            )
            .fetch_one(&mut *tx)
            .await?;
        }
    }

    let mut moved: Vec<Item> = Vec::with_capacity(item_ids.len());
    for &id in item_ids {
        let item = sqlx::query_as!(
            Item,
            r#"SELECT id AS "id!: Uuid", collection_id AS "collection_id!: Uuid",
                      number AS "number!: i64", season_number, title, poster,
                      description, file_id AS "file_id!: Uuid"
               FROM items WHERE id = ?"#,
            id,
        )
        .fetch_optional(&mut *tx)
        .await?;
        match item {
            Some(i) if i.collection_id != to_collection_id => moved.push(i),
            Some(_) => {}
            None => leptos::logging::warn!("[move_items] unknown item {id}"),
        }
    }
    if moved.is_empty() {
        tx.commit().await?;
        return Ok(());
    }

    let mut from_counts: HashMap<Uuid, i64> = HashMap::new();
    for item in &moved {
        *from_counts.entry(item.collection_id).or_insert(0) += 1;
    }

    for item in &moved {
        let next: i64 = match item.season_number {
            Some(s) => {
                sqlx::query_scalar!(
                    r#"SELECT COALESCE(MAX(number), -1) + 1 AS "n!: i64"
                       FROM items WHERE collection_id = ? AND season_number = ?"#,
                    to_collection_id,
                    s,
                )
                .fetch_one(&mut *tx)
                .await?
            }
            None => {
                sqlx::query_scalar!(
                    r#"SELECT COALESCE(MAX(number), -1) + 1 AS "n!: i64"
                       FROM items WHERE collection_id = ? AND season_number IS NULL"#,
                    to_collection_id,
                )
                .fetch_one(&mut *tx)
                .await?
            }
        };

        sqlx::query!(
            r#"UPDATE items SET collection_id = ?, number = ? WHERE id = ?"#,
            to_collection_id,
            next,
            item.id,
        )
        .execute(&mut *tx)
        .await?;
    }

    for &from_id in from_counts.keys() {
        let remaining = sqlx::query_as!(
            Item,
            r#"SELECT id AS "id!: Uuid", collection_id AS "collection_id!: Uuid",
                      number AS "number!: i64", season_number, title, poster,
                      description, file_id AS "file_id!: Uuid"
               FROM items WHERE collection_id = ?
               ORDER BY season_number ASC, number ASC, id ASC"#,
            from_id,
        )
        .fetch_all(&mut *tx)
        .await?;

        let mut counters: HashMap<Option<i64>, i64> = HashMap::new();
        for item in remaining {
            let n = counters.entry(item.season_number).or_insert(0);
            if item.number != *n {
                sqlx::query!(r#"UPDATE items SET number = ? WHERE id = ?"#, *n, item.id)
                    .execute(&mut *tx)
                    .await?;
            }
            *n += 1;
        }
    }

    async fn bump(
        tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
        kind: CollectionKind,
        id: Uuid,
        delta: i64,
    ) -> sqlx::Result<()> {
        match kind {
            CollectionKind::Video => {
                sqlx::query!(
                    r#"UPDATE video_collections SET items_count = MAX(items_count + ?, 0) WHERE id = ?"#,
                    delta, id,
                ).execute(&mut **tx).await?;
            }
            CollectionKind::Audio => {
                sqlx::query!(
                    r#"UPDATE audio_collections SET items_count = MAX(items_count + ?, 0) WHERE id = ?"#,
                    delta, id,
                ).execute(&mut **tx).await?;
            }
        }
        Ok(())
    }

    for (&from_id, &count) in &from_counts {
        bump(&mut tx, dest_kind, from_id, -count).await?;
    }
    bump(&mut tx, dest_kind, to_collection_id, moved.len() as i64).await?;

    tx.commit().await?;
    Ok(())
}

// ─── Cascade deletion ─────────────────────────────────────────────────────

pub struct ItemDeletion {
    pub media_paths: Vec<RelPath>,
    pub emptied_poster: Option<PosterUrl>,
}

pub struct BulkItemDeletion {
    pub media_paths: Vec<RelPath>,
    pub emptied_posters: Vec<PosterUrl>,
}

pub async fn delete_item_cascade(pool: &SqlitePool, id: Uuid) -> sqlx::Result<ItemDeletion> {
    let mut tx = pool.begin().await?;

    let (file_id, collection_id): (Uuid, Uuid) = sqlx::query_as!(
        _CollectionRef,
        r#"SELECT file_id AS "file_id!: Uuid", collection_id AS "collection_id!: Uuid"
           FROM items WHERE id = ?"#,
        id,
    )
    .fetch_one(&mut *tx)
    .await
    .map(|r| (r.file_id, r.collection_id))?;

    sqlx::query!(r#"DELETE FROM items WHERE id = ?"#, id)
        .execute(&mut *tx)
        .await?;

    let emptied_poster = decrement_or_delete(&mut tx, collection_id, 1).await?;
    let media_paths = purge_orphan_files(&mut tx, [file_id]).await?;

    tx.commit().await?;
    Ok(ItemDeletion {
        media_paths,
        emptied_poster,
    })
}

pub async fn delete_items_cascade(
    pool: &SqlitePool,
    item_ids: &[Uuid],
) -> sqlx::Result<BulkItemDeletion> {
    if item_ids.is_empty() {
        return Ok(BulkItemDeletion {
            media_paths: Vec::new(),
            emptied_posters: Vec::new(),
        });
    }

    let mut tx = pool.begin().await?;
    let mut file_ids: Vec<Uuid> = Vec::with_capacity(item_ids.len());
    let mut deltas: HashMap<Uuid, i64> = HashMap::new();

    for &id in item_ids {
        let row: Option<_CollectionRef> = sqlx::query_as!(
            _CollectionRef,
            r#"SELECT file_id AS "file_id!: Uuid", collection_id AS "collection_id!: Uuid"
               FROM items WHERE id = ?"#,
            id,
        )
        .fetch_optional(&mut *tx)
        .await?;

        let Some(row) = row else { continue };
        file_ids.push(row.file_id);
        *deltas.entry(row.collection_id).or_insert(0) += 1;
        sqlx::query!(r#"DELETE FROM items WHERE id = ?"#, id)
            .execute(&mut *tx)
            .await?;
    }

    let mut emptied_posters: Vec<PosterUrl> = Vec::new();
    for (collection_id, delta) in deltas {
        if let Some(poster) = decrement_or_delete(&mut tx, collection_id, delta).await? {
            emptied_posters.push(poster);
        }
    }

    let media_paths = purge_orphan_files(&mut tx, file_ids).await?;
    tx.commit().await?;
    Ok(BulkItemDeletion {
        media_paths,
        emptied_posters,
    })
}

#[derive(sqlx::FromRow)]
struct _CollectionRef {
    file_id: Uuid,
    collection_id: Uuid,
}

#[allow(dead_code)]
fn _touch(_: &mut SqliteConnection) {}
