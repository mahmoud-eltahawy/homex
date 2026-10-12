use toasty::Db;

use super::Config;
use crate::app::model::{
    Collection, CollectionRow, CollectionTag, File, Item, Section, SectionRow, SectionTag, Tag,
    canonicalize_tags, parse_search,
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

pub async fn fetch_all_tags(db: &mut Db) -> toasty::Result<Vec<Tag>> {
    Tag::all()
        .order_by(Tag::fields().name().asc())
        .exec(db)
        .await
}

/// Case-insensitive lookup of a tag by name. Returns `None` if absent.
async fn find_tag_id_by_name(db: &mut Db, name: &str) -> toasty::Result<Option<u64>> {
    let candidates = Tag::filter(Tag::fields().name().like(name.to_string()))
        .exec(db)
        .await?;
    Ok(candidates
        .into_iter()
        .find(|t| t.name.eq_ignore_ascii_case(name))
        .map(|t| t.id))
}

/// Find-or-create a tag inside a transaction.
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

/// Resolve a set of `tag_id`s to sorted `Tag` rows.
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
    let ids: Vec<u64> = SectionTag::filter(SectionTag::fields().section_id().eq(section_id))
        .exec(db)
        .await?
        .into_iter()
        .map(|j| j.tag_id)
        .collect();
    tags_for_joins(db, &ids).await
}

pub async fn fetch_collection_tags(db: &mut Db, collection_id: u64) -> toasty::Result<Vec<Tag>> {
    let ids: Vec<u64> =
        CollectionTag::filter(CollectionTag::fields().collection_id().eq(collection_id))
            .exec(db)
            .await?
            .into_iter()
            .map(|j| j.tag_id)
            .collect();
    tags_for_joins(db, &ids).await
}

pub async fn set_section_tags(
    db: &mut Db,
    section_id: u64,
    names: &[String],
) -> toasty::Result<()> {
    let normalized = canonicalize_tags(names);
    let mut tx = db.transaction().await?;

    for row in SectionTag::filter(SectionTag::fields().section_id().eq(section_id))
        .exec(&mut tx)
        .await?
    {
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

    for row in CollectionTag::filter(CollectionTag::fields().collection_id().eq(collection_id))
        .exec(&mut tx)
        .await?
    {
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

// ─── Tag-driven collection filtering ──────────────────────────────────────

/// Collection ids carrying **every** tag in `tag_names` (case-insensitive).
/// Returns empty if any listed tag doesn't exist.
async fn collection_ids_with_all_tags(
    db: &mut Db,
    tag_names: &[String],
) -> toasty::Result<Vec<u64>> {
    let mut intersection: Option<Vec<u64>> = None;

    for name in tag_names {
        let Some(tag_id) = find_tag_id_by_name(db, name).await? else {
            return Ok(Vec::new());
        };
        let ids: Vec<u64> = CollectionTag::filter(CollectionTag::fields().tag_id().eq(tag_id))
            .exec(db)
            .await?
            .into_iter()
            .map(|j| j.collection_id)
            .collect();

        intersection = Some(match intersection {
            None => ids,
            Some(prev) => prev.into_iter().filter(|id| ids.contains(id)).collect(),
        });
        if intersection.as_ref().is_some_and(Vec::is_empty) {
            return Ok(Vec::new());
        }
    }

    let mut ids = intersection.unwrap_or_default();
    ids.sort_unstable();
    ids.dedup();
    Ok(ids)
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
        let mut s: Section = row.into();
        s.tags = fetch_section_tags(db, s.id).await?;
        out.push(s);
    }
    Ok(out)
}

pub async fn fetch_section_by_slug(db: &mut Db, slug: &str) -> toasty::Result<Option<Section>> {
    let Some(row) = SectionRow::filter(SectionRow::fields().slug().eq(slug.to_string()))
        .first()
        .exec(db)
        .await?
    else {
        return Ok(None);
    };
    let mut s: Section = row.into();
    s.tags = fetch_section_tags(db, s.id).await?;
    Ok(Some(s))
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
    for row in SectionTag::filter(SectionTag::fields().section_id().eq(id as u64))
        .exec(db)
        .await?
    {
        row.delete().exec(db).await?;
    }
    SectionRow::get_by_id(db, &(id as u64))
        .await?
        .delete()
        .exec(db)
        .await
}

// ─── Collection queries ───────────────────────────────────────────────────

pub async fn fetch_collections(
    db: &mut Db,
    section_slug: &str,
    offset: usize,
    size: usize,
    search: Option<&str>,
) -> toasty::Result<Vec<Collection>> {
    let parsed = search.map(parse_search).unwrap_or_default();

    let tag_ids: Option<Vec<u64>> = if parsed.tags.is_empty() {
        None
    } else {
        Some(collection_ids_with_all_tags(db, &parsed.tags).await?)
    };
    if tag_ids.as_ref().is_some_and(Vec::is_empty) {
        return Ok(Vec::new());
    }

    let rows = if let Some(ids) = &tag_ids {
        // Tag filter crosses the join table → fetch, filter, paginate in Rust.
        let mut q = CollectionRow::filter(
            CollectionRow::fields()
                .section_slug()
                .eq(section_slug.to_string()),
        );
        if let Some(t) = &parsed.text {
            q = q.filter(CollectionRow::fields().title().like(format!("%{t}%")));
        }
        q.order_by((
            CollectionRow::fields().position().asc(),
            CollectionRow::fields().title().asc(),
            CollectionRow::fields().id().asc(),
        ))
        .exec(db)
        .await?
        .into_iter()
        .filter(|r| ids.contains(&r.id))
        .skip(offset)
        .take(size)
        .collect()
    } else {
        let mut q = CollectionRow::filter(
            CollectionRow::fields()
                .section_slug()
                .eq(section_slug.to_string()),
        );
        if let Some(t) = &parsed.text {
            q = q.filter(CollectionRow::fields().title().like(format!("%{t}%")));
        }
        q.order_by((
            CollectionRow::fields().position().asc(),
            CollectionRow::fields().title().asc(),
            CollectionRow::fields().id().asc(),
        ))
        .limit(size)
        .offset(offset)
        .exec(db)
        .await?
    };

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        let mut c: Collection = row.into();
        c.tags = fetch_collection_tags(db, c.id).await?;
        out.push(c);
    }
    Ok(out)
}

pub async fn fetch_collections_count(
    db: &mut Db,
    section_slug: &str,
    search: Option<&str>,
) -> toasty::Result<i64> {
    let parsed = search.map(parse_search).unwrap_or_default();

    let tag_ids: Option<Vec<u64>> = if parsed.tags.is_empty() {
        None
    } else {
        Some(collection_ids_with_all_tags(db, &parsed.tags).await?)
    };
    if tag_ids.as_ref().is_some_and(Vec::is_empty) {
        return Ok(0);
    }

    let mut q = CollectionRow::filter(
        CollectionRow::fields()
            .section_slug()
            .eq(section_slug.to_string()),
    );
    if let Some(t) = &parsed.text {
        q = q.filter(CollectionRow::fields().title().like(format!("%{t}%")));
    }

    match tag_ids {
        None => Ok(q.count().exec(db).await? as i64),
        Some(ids) => {
            let n = q
                .exec(db)
                .await?
                .into_iter()
                .filter(|r| ids.contains(&r.id))
                .count();
            Ok(n as i64)
        }
    }
}

pub async fn fetch_collection_detail(
    db: &mut Db,
    section_slug: &str,
    collection_id: i64,
) -> toasty::Result<Option<Collection>> {
    let Some(row) = CollectionRow::filter(
        CollectionRow::fields()
            .section_slug()
            .eq(section_slug.to_string())
            .and(CollectionRow::fields().id().eq(collection_id as u64)),
    )
    .first()
    .exec(db)
    .await?
    else {
        return Ok(None);
    };
    let mut c: Collection = row.into();
    c.tags = fetch_collection_tags(db, c.id).await?;
    Ok(Some(c))
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
        // `title` is NOT NULL, so the update setter wants `String`, not
        // `Option<String>`. The server fn validates before calling.
        CollectionField::Title => {
            row.update()
                .title(value.unwrap_or_default().to_string())
                .exec(db)
                .await
        }
        // `description` and `poster` are nullable; `Option<String>` is
        // accepted directly, and `None` assigns SQL NULL.
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

pub async fn update_item_title(db: &mut Db, id: i64, title: Option<&str>) -> toasty::Result<()> {
    let mut item = Item::get_by_id(db, &(id as u64)).await?;
    item.update().title(title.map(String::from)).exec(db).await
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

/// For each file id, delete the file row if no item still references it,
/// and return the relative paths so the caller can unlink the media on disk.
async fn purge_orphan_files(
    tx: &mut toasty::Transaction<'_>,
    file_ids: impl IntoIterator<Item = u64>,
) -> toasty::Result<Vec<String>> {
    let mut paths = Vec::new();
    for fid in file_ids {
        let still_used = !Item::filter(Item::fields().file_id().eq(fid))
            .limit(1)
            .exec(tx)
            .await?
            .is_empty();
        if still_used {
            continue;
        }
        if let Ok(f) = File::get_by_id(tx, &fid).await {
            paths.push(f.relative_path.clone());
            f.delete().exec(tx).await?;
        }
    }
    Ok(paths)
}

pub async fn delete_collection_cascade(db: &mut Db, id: i64) -> toasty::Result<CollectionDeletion> {
    let mut tx = db.transaction().await?;

    let collection = CollectionRow::get_by_id(&mut tx, &(id as u64)).await?;
    let poster_url = collection.poster.clone();

    let items = Item::filter(Item::fields().collection_id().eq(id as u64))
        .exec(&mut tx)
        .await?;
    let file_ids: Vec<u64> = items.iter().map(|i| i.file_id).collect();

    // Drop the collection↔tag joins. Tag rows themselves are shared and stay.
    for row in CollectionTag::filter(CollectionTag::fields().collection_id().eq(id as u64))
        .exec(&mut tx)
        .await?
    {
        row.delete().exec(&mut tx).await?;
    }

    collection.clone().delete().exec(&mut tx).await?;
    for item in items {
        item.delete().exec(&mut tx).await?;
    }

    let media_paths = purge_orphan_files(&mut tx, file_ids).await?;

    tx.commit().await?;
    Ok(CollectionDeletion {
        media_paths,
        poster_url,
    })
}

pub async fn delete_item_cascade(db: &mut Db, id: i64) -> toasty::Result<ItemDeletion> {
    let mut tx = db.transaction().await?;

    let item = Item::get_by_id(&mut tx, &(id as u64)).await?;
    let (file_id, collection_id) = (item.file_id, item.collection_id);
    item.delete().exec(&mut tx).await?;

    // Decrement the denormalized counter on the parent collection.
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

pub struct CollectionMove {
    pub from_slug: String,
    pub poster_url: Option<String>,
}

pub async fn move_collection_to_section(
    db: &mut Db,
    collection_id: i64,
    to_section_slug: &str,
) -> toasty::Result<CollectionMove> {
    let to_section =
        SectionRow::filter(SectionRow::fields().slug().eq(to_section_slug.to_string()))
            .first()
            .exec(db)
            .await?
            .ok_or_else(|| err_not_found("section"))?;

    let mut c = CollectionRow::get_by_id(db, &(collection_id as u64)).await?;
    let from_slug = c.section_slug.clone();
    let poster_url = c.poster.clone();

    c.update()
        .section_id(to_section.id)
        .section_slug(to_section.slug.clone())
        .exec(db)
        .await?;

    Ok(CollectionMove {
        from_slug,
        poster_url,
    })
}
