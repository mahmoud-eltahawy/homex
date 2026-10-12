use std::collections::HashMap;

use toasty::Db;
use uuid::Uuid;

use super::files::purge_orphan_files;
use crate::app::model::{CollectionRow, Item};

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
    if item_ids.is_empty() {
        return Ok(());
    }

    let mut tx = db.transaction().await?;

    let _ = CollectionRow::get_by_id(&mut tx, &to_collection_id).await?;

    let mut moved: Vec<Item> = Vec::with_capacity(item_ids.len());
    for &id in item_ids {
        if let Ok(item) = Item::get_by_id(&mut tx, &id).await
            && item.collection_id != to_collection_id
        {
            moved.push(item);
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

    for (&from_id, &count) in &from_counts {
        let mut c = CollectionRow::get_by_id(&mut tx, &from_id).await?;
        let items_count = c.items_count;
        c.update()
            .items_count((items_count - count).max(0))
            .exec(&mut tx)
            .await?;
    }
    let mut to_c = CollectionRow::get_by_id(&mut tx, &to_collection_id).await?;
    let items_count = to_c.items_count;
    to_c.update()
        .items_count(items_count + moved.len() as i64)
        .exec(&mut tx)
        .await?;

    tx.commit().await?;
    Ok(())
}

// ─── Cascade deletion ─────────────────────────────────────────────────────

pub struct ItemDeletion {
    pub media_paths: Vec<String>,
}

pub async fn delete_item_cascade(db: &mut Db, id: Uuid) -> toasty::Result<ItemDeletion> {
    let mut tx = db.transaction().await?;

    let item = Item::get_by_id(&mut tx, &id).await?;
    let (file_id, collection_id) = (item.file_id, item.collection_id);
    item.delete().exec(&mut tx).await?;

    let mut collection = CollectionRow::get_by_id(&mut tx, &collection_id).await?;
    let items_count = collection.items_count;
    collection
        .update()
        .items_count((items_count - 1).max(0))
        .exec(&mut tx)
        .await?;

    let media_paths = purge_orphan_files(&mut tx, [file_id]).await?;

    tx.commit().await?;
    Ok(ItemDeletion { media_paths })
}
