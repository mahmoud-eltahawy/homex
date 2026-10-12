use toasty::Db;

use super::Config;
use crate::app::model::{
    Collection, CollectionRow, CollectionTag, File, Item, Section, SectionRow, SectionTag, Tag,
    canonicalize_tags,
};

// ─── Pool init ────────────────────────────────────────────────────────────

pub async fn init(config: &Config) -> Result<Db, Box<dyn std::error::Error>> {
    tokio::fs::create_dir_all(&config.storage.data_dir).await?;

    let db_path = config.db_path();
    let is_fresh = !db_path.exists();
    let force_reset = std::env::var("HOMEX_RESET_SCHEMA").is_ok();

    let url = format!("sqlite:{}", db_path.display());
    let db = toasty::Db::builder()
        .models(toasty::models!(
            crate::app::model::File,
            crate::app::model::SectionRow,
            crate::app::model::CollectionRow,
            crate::app::model::Item,
            crate::app::model::Tag,
            crate::app::model::SectionTag,
            crate::app::model::CollectionTag,
        ))
        .connect(&url)
        .await?;

    if is_fresh || force_reset {
        leptos::logging::log!("[db] pushing schema (fresh={is_fresh}, force={force_reset})");
        db.push_schema().await?;
    } else {
        leptos::logging::log!("[db] schema exists, skipping push_schema");
    }

    Ok(db)
}

fn err_not_found(what: &str) -> toasty::Error {
    toasty::Error::record_not_found(what)
}

// ─── Tags ─────────────────────────────────────────────────────────────────

/// Find a tag by name, or create it. Called inside a tag-set transaction.
async fn upsert_tag_tx(tx: &mut toasty::Transaction<'_>, name: &str) -> toasty::Result<Tag> {
    if let Some(t) = Tag::filter(Tag::fields().name().eq(name.to_string()))
        .first()
        .exec(tx)
        .await?
    {
        return Ok(t);
    }
    toasty::create!(Tag {
        name: name.to_string(),
    })
    .exec(tx)
    .await
}

async fn tags_for_joins(db: &mut Db, tag_ids: &[u64]) -> toasty::Result<Vec<Tag>> {
    let mut out = Vec::with_capacity(tag_ids.len());
    for id in tag_ids {
        if let Ok(t) = Tag::get_by_id(db, id).await {
            out.push(t);
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub async fn fetch_section_tags(db: &mut Db, section_id: u64) -> toasty::Result<Vec<Tag>> {
    let joins = SectionTag::filter(SectionTag::fields().section_id().eq(section_id))
        .exec(db)
        .await?;
    let ids: Vec<u64> = joins.iter().map(|j| j.tag_id).collect();
    tags_for_joins(db, &ids).await
}

pub async fn fetch_collection_tags(db: &mut Db, collection_id: u64) -> toasty::Result<Vec<Tag>> {
    let joins = CollectionTag::filter(CollectionTag::fields().collection_id().eq(collection_id))
        .exec(db)
        .await?;
    let ids: Vec<u64> = joins.iter().map(|j| j.tag_id).collect();
    tags_for_joins(db, &ids).await
}

pub async fn set_section_tags(
    db: &mut Db,
    section_id: u64,
    names: &[String],
) -> toasty::Result<()> {
    let normalized = canonicalize_tags(names);
    let mut tx = db.transaction().await?;

    let existing = SectionTag::filter(SectionTag::fields().section_id().eq(section_id))
        .exec(&mut tx)
        .await?;
    for row in existing {
        row.delete().exec(&mut tx).await?;
    }

    for name in &normalized {
        let tag = upsert_tag_tx(&mut tx, name).await?;
        toasty::create!(SectionTag {
            section_id,
            tag_id: tag.id,
        })
        .exec(&mut tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn set_collection_tags(
    db: &mut Db,
    collection_id: u64,
    names: &[String],
) -> toasty::Result<()> {
    let normalized = canonicalize_tags(names);
    let mut tx = db.transaction().await?;

    let existing = CollectionTag::filter(CollectionTag::fields().collection_id().eq(collection_id))
        .exec(&mut tx)
        .await?;
    for row in existing {
        row.delete().exec(&mut tx).await?;
    }

    for name in &normalized {
        let tag = upsert_tag_tx(&mut tx, name).await?;
        toasty::create!(CollectionTag {
            collection_id,
            tag_id: tag.id,
        })
        .exec(&mut tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

// ─── Section queries ──────────────────────────────────────────────────────

pub async fn fetch_sections(db: &mut Db) -> toasty::Result<Vec<Section>> {
    let rows = SectionRow::all()
        .order_by((
            SectionRow::fields().position().asc(),
            SectionRow::fields().id().asc(),
        ))
        .exec(db)
        .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut section: Section = row.into();
        section.tags = fetch_section_tags(db, section.id).await?;
        out.push(section);
    }
    Ok(out)
}

pub async fn fetch_section_by_slug(db: &mut Db, slug: &str) -> toasty::Result<Option<Section>> {
    let row = SectionRow::filter(SectionRow::fields().slug().eq(slug.to_string()))
        .first()
        .exec(db)
        .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let mut section: Section = row.into();
    section.tags = fetch_section_tags(db, section.id).await?;
    Ok(Some(section))
}

pub async fn insert_section(
    db: &mut Db,
    slug: &str,
    title: &str,
    media_kind: &str,
    nested: bool,
) -> toasty::Result<u64> {
    let next = next_section_position(db).await?;
    let row = toasty::create!(SectionRow {
        slug: slug.to_string(),
        title: title.to_string(),
        media_kind: media_kind.to_string(),
        nested,
        position: next,
    })
    .exec(db)
    .await?;
    Ok(row.id)
}

async fn next_section_position(db: &mut Db) -> toasty::Result<i64> {
    let last = SectionRow::all()
        .order_by(SectionRow::fields().position().desc())
        .limit(1)
        .exec(db)
        .await?;
    Ok(last.first().map(|s| s.position + 1).unwrap_or(0))
}

pub async fn update_section(
    db: &mut Db,
    id: i64,
    title: &str,
    media_kind: &str,
    nested: bool,
) -> toasty::Result<()> {
    let mut row = SectionRow::get_by_id(db, &(id as u64)).await?;
    row.update()
        .title(title.to_string())
        .media_kind(media_kind.to_string())
        .nested(nested)
        .exec(db)
        .await
}

pub async fn update_section_title(db: &mut Db, id: i64, title: &str) -> toasty::Result<()> {
    let mut row = SectionRow::get_by_id(db, &(id as u64)).await?;
    row.update().title(title.to_string()).exec(db).await
}

pub async fn delete_section(db: &mut Db, id: i64) -> toasty::Result<()> {
    // Drop join rows first so no dangling `section_tags` survive.
    let joins = SectionTag::filter(SectionTag::fields().section_id().eq(id as u64))
        .exec(db)
        .await?;
    for row in joins {
        row.delete().exec(db).await?;
    }

    let row = SectionRow::get_by_id(db, &(id as u64)).await?;
    row.delete().exec(db).await
}

// ─── Collection queries ───────────────────────────────────────────────────

pub async fn fetch_collections(
    db: &mut Db,
    section_slug: &str,
    offset: usize,
    size: usize,
    search: Option<&str>,
) -> toasty::Result<Vec<Collection>> {
    let mut q = CollectionRow::filter(
        CollectionRow::fields()
            .section_slug()
            .eq(section_slug.to_string()),
    );
    if let Some(s) = search {
        q = q.filter(CollectionRow::fields().title().like(format!("%{s}%")));
    }
    let rows = q
        .order_by((
            CollectionRow::fields().position().asc(),
            CollectionRow::fields().title().asc(),
            CollectionRow::fields().id().asc(),
        ))
        .limit(size)
        .offset(offset)
        .exec(db)
        .await?;

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut collection: Collection = row.into();
        collection.tags = fetch_collection_tags(db, collection.id).await?;
        out.push(collection);
    }
    Ok(out)
}

pub async fn fetch_collections_count(
    db: &mut Db,
    section_slug: &str,
    search: Option<&str>,
) -> toasty::Result<i64> {
    let mut q = CollectionRow::filter(
        CollectionRow::fields()
            .section_slug()
            .eq(section_slug.to_string()),
    );
    if let Some(s) = search {
        q = q.filter(CollectionRow::fields().title().like(format!("%{s}%")));
    }
    Ok(q.count().exec(db).await? as i64)
}

pub async fn fetch_collection_detail(
    db: &mut Db,
    section_slug: &str,
    collection_id: i64,
) -> toasty::Result<Option<Collection>> {
    let row = CollectionRow::filter(
        CollectionRow::fields()
            .section_slug()
            .eq(section_slug.to_string())
            .and(CollectionRow::fields().id().eq(collection_id as u64)),
    )
    .first()
    .exec(db)
    .await?;

    let Some(row) = row else {
        return Ok(None);
    };

    let mut collection: Collection = row.into();
    collection.tags = fetch_collection_tags(db, collection.id).await?;
    Ok(Some(collection))
}

pub async fn insert_empty_collection(db: &mut Db, section_slug: &str) -> toasty::Result<u64> {
    let section = SectionRow::filter(SectionRow::fields().slug().eq(section_slug.to_string()))
        .first()
        .exec(db)
        .await?
        .ok_or_else(|| err_not_found("section"))?;

    let next = next_collection_position(db, section.id).await?;
    let row = toasty::create!(CollectionRow {
        section_id: section.id,
        section_slug: section.slug.clone(),
        title: "بدون عنوان".to_string(),
        poster: None,
        description: None,
        position: next,
        items_count: 0,
    })
    .exec(db)
    .await?;
    Ok(row.id)
}

async fn next_collection_position(db: &mut Db, section_id: u64) -> toasty::Result<i64> {
    let last = CollectionRow::filter(CollectionRow::fields().section_id().eq(section_id))
        .order_by(CollectionRow::fields().position().desc())
        .limit(1)
        .exec(db)
        .await?;
    Ok(last.first().map(|c| c.position + 1).unwrap_or(0))
}

pub enum CollectionField {
    Title,
    Description,
    Poster,
    Tags,
}

pub async fn update_collection_field(
    db: &mut Db,
    id: i64,
    field: CollectionField,
    value: Option<&str>,
) -> toasty::Result<()> {
    // Tags live in `collection_tags`, not on the row.
    if matches!(field, CollectionField::Tags) {
        let names: Vec<String> = value
            .unwrap_or_default()
            .split(',')
            .map(str::to_string)
            .collect();
        return set_collection_tags(db, id as u64, &names).await;
    }

    let mut row = CollectionRow::get_by_id(db, &(id as u64)).await?;
    match field {
        CollectionField::Title => {
            row.update()
                .title(value.unwrap_or_default().to_string())
                .exec(db)
                .await
        }
        CollectionField::Description => {
            row.update()
                .description(value.map(String::from))
                .exec(db)
                .await
        }
        CollectionField::Poster => row.update().poster(value.map(String::from)).exec(db).await,
        CollectionField::Tags => unreachable!("handled above"),
    }
}

pub async fn delete_collection(db: &mut Db, id: i64) -> toasty::Result<()> {
    let row = CollectionRow::get_by_id(db, &(id as u64)).await?;
    row.delete().exec(db).await
}

// ─── Item queries ─────────────────────────────────────────────────────────

pub async fn fetch_items(db: &mut Db, collection_id: i64) -> toasty::Result<Vec<Item>> {
    Item::filter(Item::fields().collection_id().eq(collection_id as u64))
        .order_by((
            Item::fields().season_number().asc(),
            Item::fields().number().asc(),
            Item::fields().id().asc(),
        ))
        .exec(db)
        .await
}

pub async fn next_item_number(
    db: &mut Db,
    collection_id: i64,
    season_number: Option<i64>,
) -> toasty::Result<i64> {
    let mut q = Item::filter(Item::fields().collection_id().eq(collection_id as u64));
    q = match season_number {
        Some(s) => q.filter(Item::fields().season_number().eq(s)),
        None => q.filter(Item::fields().season_number().is_none()),
    };
    let last = q
        .order_by(Item::fields().number().desc())
        .limit(1)
        .exec(db)
        .await?;
    Ok(last.first().map(|i| i.number + 1).unwrap_or(0))
}

pub async fn insert_item(
    db: &mut Db,
    collection_id: i64,
    number: i64,
    season_number: Option<i64>,
    title: Option<&str>,
    file_id: i64,
) -> toasty::Result<u64> {
    let i = toasty::create!(Item {
        collection_id: collection_id as u64,
        number,
        season_number,
        title: title.map(String::from),
        poster: None,
        description: None,
        file_id: file_id as u64,
    })
    .exec(db)
    .await?;
    Ok(i.id)
}

pub async fn update_item_title(db: &mut Db, id: i64, title: Option<&str>) -> toasty::Result<()> {
    let mut item = Item::get_by_id(db, &(id as u64)).await?;
    item.update().title(title.map(String::from)).exec(db).await
}

pub async fn delete_item(db: &mut Db, id: i64) -> toasty::Result<()> {
    let item = Item::get_by_id(db, &(id as u64)).await?;
    item.delete().exec(db).await
}

// ─── File queries ─────────────────────────────────────────────────────────

pub async fn insert_file(
    db: &mut Db,
    relative_path: &str,
    size_bytes: i64,
    duration_secs: i64,
) -> toasty::Result<u64> {
    let f = toasty::create!(File {
        relative_path: relative_path.to_string(),
        size_bytes,
        duration_secs,
    })
    .exec(db)
    .await?;
    Ok(f.id)
}

pub async fn fetch_file_path(db: &mut Db, file_id: i64) -> toasty::Result<Option<String>> {
    Ok(File::get_by_id(db, &(file_id as u64))
        .await
        .ok()
        .map(|f| f.relative_path))
}

// ─── Upload-support ───────────────────────────────────────────────────────

pub async fn fetch_section_kind(db: &mut Db, slug: &str) -> toasty::Result<Option<String>> {
    Ok(
        SectionRow::filter(SectionRow::fields().slug().eq(slug.to_string()))
            .first()
            .exec(db)
            .await?
            .map(|s| s.media_kind),
    )
}

// ─── Cascade deletion ─────────────────────────────────────────────────────

pub struct CollectionDeletion {
    pub media_paths: Vec<String>,
    pub poster_url: Option<String>,
}

pub struct ItemDeletion {
    pub media_paths: Vec<String>,
}

pub async fn delete_collection_cascade(db: &mut Db, id: i64) -> toasty::Result<CollectionDeletion> {
    let mut tx = db.transaction().await?;

    let collection = CollectionRow::get_by_id(&mut tx, &(id as u64)).await?;
    let poster_url = collection.poster.clone();

    let items = Item::filter(Item::fields().collection_id().eq(id as u64))
        .exec(&mut tx)
        .await?;
    let file_ids: Vec<u64> = items.iter().map(|i| i.file_id).collect();

    // Drop collection↔tag join rows. The `tags` rows themselves stay — they
    // may be referenced by other sections/collections.
    let joins = CollectionTag::filter(CollectionTag::fields().collection_id().eq(id as u64))
        .exec(&mut tx)
        .await?;
    for row in joins {
        row.delete().exec(&mut tx).await?;
    }

    collection.clone().delete().exec(&mut tx).await?;
    for item in items {
        item.delete().exec(&mut tx).await?;
    }

    let mut media_paths = Vec::new();
    for fid in file_ids {
        let still_used = !Item::filter(Item::fields().file_id().eq(fid))
            .limit(1)
            .exec(&mut tx)
            .await?
            .is_empty();
        if still_used {
            continue;
        }
        if let Ok(f) = File::get_by_id(&mut tx, &fid).await {
            media_paths.push(f.relative_path.clone());
            f.delete().exec(&mut tx).await?;
        }
    }

    tx.commit().await?;
    Ok(CollectionDeletion {
        media_paths,
        poster_url,
    })
}

pub async fn delete_item_cascade(db: &mut Db, id: i64) -> toasty::Result<ItemDeletion> {
    let mut tx = db.transaction().await?;

    let item = Item::get_by_id(&mut tx, &(id as u64)).await?;
    let file_id = item.file_id;
    let collection_id = item.collection_id;
    item.delete().exec(&mut tx).await?;

    // Decrement the denormalized counter on the parent collection.
    let mut collection = CollectionRow::get_by_id(&mut tx, &collection_id).await?;
    let next_count = (collection.items_count - 1).max(0);
    collection
        .update()
        .items_count(next_count)
        .exec(&mut tx)
        .await?;

    let mut media_paths = Vec::new();
    let still_used = !Item::filter(Item::fields().file_id().eq(file_id))
        .limit(1)
        .exec(&mut tx)
        .await?
        .is_empty();
    if !still_used && let Ok(f) = File::get_by_id(&mut tx, &file_id).await {
        media_paths.push(f.relative_path.clone());
        f.delete().exec(&mut tx).await?;
    }

    tx.commit().await?;
    Ok(ItemDeletion { media_paths })
}

// ─── Reordering ───────────────────────────────────────────────────────────

pub async fn swap_item_order(db: &mut Db, id: i64, is_up: bool) -> toasty::Result<()> {
    let mut tx = db.transaction().await?;

    let item = Item::get_by_id(&mut tx, &(id as u64)).await?;

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

    let neighbour = q.limit(1).exec(&mut tx).await?;
    let Some(neighbour) = neighbour.into_iter().next() else {
        return Ok(());
    };

    let a_number = item.number;
    let b_number = neighbour.number;

    let mut a = Item::get_by_id(&mut tx, &item.id).await?;
    a.update().number(b_number).exec(&mut tx).await?;

    let mut b = Item::get_by_id(&mut tx, &neighbour.id).await?;
    b.update().number(a_number).exec(&mut tx).await?;

    tx.commit().await?;
    Ok(())
}
