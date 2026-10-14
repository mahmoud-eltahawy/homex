use std::collections::HashMap;

use toasty::Db;
use uuid::Uuid;

use super::collections::decrement_or_delete;
use super::files::purge_orphan_files;
use crate::app::model::Item;

// ─── Item queries ─────────────────────────────────────────────────────────

pub async fn fetch_items(db: &mut Db, collection_id: Uuid) -> toasty::Result<Vec<Item>> {
    Item::filter(Item::fields().collection_id().eq(collection_id))
        .order_by((
            Item::fields().season_number().asc(),
            Item::fields().number().asc(),
            Item::fields().id().asc(),
        ))
        .exec(db)
        .await
}

pub async fn update_item_title(db: &mut Db, id: Uuid, title: Option<&str>) -> toasty::Result<()> {
    let mut item = Item::get_by_id(db, &id).await?;
    item.update().title(title.map(String::from)).exec(db).await
}

// ─── Reordering ───────────────────────────────────────────────────────────

pub async fn swap_item_order(db: &mut Db, id: Uuid, is_up: bool) -> toasty::Result<()> {
    let mut tx = db.transaction().await?;

    let item = Item::get_by_id(&mut tx, &id).await?;

    let mut q = Item::filter(Item::fields().collection_id().eq(item.collection_id));
    q = match item.season_number {
        Some(s) => q.filter(Item::fields().season_number().eq(s)),
        None => q.filter(Item::fields().season_number().is_none()),
    };
    q = if is_up {
        q.filter(Item::fields().number().lt(item.number))
            .order_by(Item::fields().number().desc())
    } else {
        q.filter(Item::fields().number().gt(item.number))
            .order_by(Item::fields().number().asc())
    };

    let Some(neighbour) = q.limit(1).exec(&mut tx).await?.into_iter().next() else {
        return Ok(());
    };

    let (a_number, b_number) = (item.number, neighbour.number);

    let mut a = Item::get_by_id(&mut tx, &item.id).await?;
    a.update().number(b_number).exec(&mut tx).await?;

    let mut b = Item::get_by_id(&mut tx, &neighbour.id).await?;
    b.update().number(a_number).exec(&mut tx).await?;

    tx.commit().await?;
    Ok(())
}

// ─── Move items between collections ───────────────────────────────────────

pub async fn move_items_to_collection(
    db: &mut Db,
    item_ids: &[Uuid],
    to_collection_id: Uuid,
) -> toasty::Result<()> {
    use super::collections::kind_of;
    use crate::app::model::{AudioCollectionRow, CollectionKind, VideoCollectionRow};

    if item_ids.is_empty() {
        return Ok(());
    }

    let dest_kind = kind_of(db, to_collection_id)
        .await?
        .ok_or_else(|| super::err_not_found("collection"))?;

    let mut tx = db.transaction().await?;

    // Verify destination exists.
    match dest_kind {
        CollectionKind::Video => {
            VideoCollectionRow::get_by_id(&mut tx, &to_collection_id).await?;
        }
        CollectionKind::Audio => {
            AudioCollectionRow::get_by_id(&mut tx, &to_collection_id).await?;
        }
    }

    let mut moved: Vec<Item> = Vec::with_capacity(item_ids.len());
    for &id in item_ids {
        let item = Item::filter(Item::fields().id().eq(id))
            .first()
            .exec(&mut tx)
            .await?;
        match item {
            Some(i) if i.collection_id != to_collection_id => moved.push(i),
            Some(_) => {} // already in destination — idempotent no-op
            None => {
                leptos::logging::warn!("[move_items] unknown item {id}");
            }
        }
    }
    if moved.is_empty() {
        return Ok(());
    }

    let mut from_counts: HashMap<Uuid, i64> = HashMap::new();
    for item in &moved {
        *from_counts.entry(item.collection_id).or_insert(0) += 1;
    }

    for item in &moved {
        let mut q = Item::filter(Item::fields().collection_id().eq(to_collection_id));
        q = match item.season_number {
            Some(s) => q.filter(Item::fields().season_number().eq(s)),
            None => q.filter(Item::fields().season_number().is_none()),
        };
        let next = q
            .order_by(Item::fields().number().desc())
            .limit(1)
            .exec(&mut tx)
            .await?
            .first()
            .map(|i| i.number + 1)
            .unwrap_or(0);

        let mut m = Item::get_by_id(&mut tx, &item.id).await?;
        m.update()
            .collection_id(to_collection_id)
            .number(next)
            .exec(&mut tx)
            .await?;
    }

    // Renumber sources per season.
    for &from_id in from_counts.keys() {
        let remaining = Item::filter(Item::fields().collection_id().eq(from_id))
            .order_by((
                Item::fields().season_number().asc(),
                Item::fields().number().asc(),
                Item::fields().id().asc(),
            ))
            .exec(&mut tx)
            .await?;

        let mut counters: HashMap<Option<i64>, i64> = HashMap::new();
        for item in remaining {
            let n = counters.entry(item.season_number).or_insert(0);
            if item.number != *n {
                let mut m = Item::get_by_id(&mut tx, &item.id).await?;
                m.update().number(*n).exec(&mut tx).await?;
            }
            *n += 1;
        }
    }

    // Bump items_count on both source and destination.
    async fn bump(
        tx: &mut toasty::Transaction<'_>,
        kind: CollectionKind,
        id: Uuid,
        delta: i64,
    ) -> toasty::Result<()> {
        match kind {
            CollectionKind::Video => {
                let mut c = VideoCollectionRow::get_by_id(tx, &id).await?;
                let n = (c.items_count + delta).max(0);
                c.update().items_count(n).exec(tx).await
            }
            CollectionKind::Audio => {
                let mut c = AudioCollectionRow::get_by_id(tx, &id).await?;
                let n = (c.items_count + delta).max(0);
                c.update().items_count(n).exec(tx).await
            }
        }
    }

    for (&from_id, &count) in &from_counts {
        // Sources are always the same kind as the destination.
        bump(&mut tx, dest_kind, from_id, -count).await?;
    }
    bump(&mut tx, dest_kind, to_collection_id, moved.len() as i64).await?;

    tx.commit().await?;
    Ok(())
}

// ─── Cascade deletion ─────────────────────────────────────────────────────

pub struct ItemDeletion {
    pub media_paths: Vec<String>,
    pub emptied_poster: Option<String>,
}

pub struct BulkItemDeletion {
    pub media_paths: Vec<String>,
    pub emptied_posters: Vec<String>,
}

pub async fn delete_item_cascade(db: &mut Db, id: Uuid) -> toasty::Result<ItemDeletion> {
    let mut tx = db.transaction().await?;

    let item = Item::get_by_id(&mut tx, &id).await?;
    let (file_id, collection_id) = (item.file_id, item.collection_id);
    item.delete().exec(&mut tx).await?;

    let emptied_poster = decrement_or_delete(&mut tx, collection_id, 1).await?;
    let media_paths = purge_orphan_files(&mut tx, [file_id]).await?;

    tx.commit().await?;
    Ok(ItemDeletion {
        media_paths,
        emptied_poster,
    })
}

pub async fn delete_items_cascade(
    db: &mut Db,
    item_ids: &[Uuid],
) -> toasty::Result<BulkItemDeletion> {
    if item_ids.is_empty() {
        return Ok(BulkItemDeletion {
            media_paths: Vec::new(),
            emptied_posters: Vec::new(),
        });
    }

    let mut tx = db.transaction().await?;
    let mut file_ids: Vec<Uuid> = Vec::with_capacity(item_ids.len());
    let mut deltas: HashMap<Uuid, i64> = HashMap::new();

    for &id in item_ids {
        let item = match Item::filter(Item::fields().id().eq(id))
            .first()
            .exec(&mut tx)
            .await?
        {
            Some(i) => i,
            None => continue,
        };
        file_ids.push(item.file_id);
        *deltas.entry(item.collection_id).or_insert(0) += 1;
        item.delete().exec(&mut tx).await?;
    }

    let mut emptied_posters: Vec<String> = Vec::new();
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
