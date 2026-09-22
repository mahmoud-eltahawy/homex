use toasty::Db;

use super::Config;
use crate::app::model::{Collection, File, Item, Section};

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
            crate::app::model::Section,
            crate::app::model::Collection,
            crate::app::model::Item,
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

// ─── Section queries ──────────────────────────────────────────────────────

pub async fn fetch_sections(db: &mut Db) -> toasty::Result<Vec<Section>> {
    Section::all()
        .order_by((
            Section::fields().position().asc(),
            Section::fields().id().asc(),
        ))
        .exec(db)
        .await
}

pub async fn fetch_section_by_slug(db: &mut Db, slug: &str) -> toasty::Result<Option<Section>> {
    Section::filter(Section::fields().slug().eq(slug.to_string()))
        .first()
        .exec(db)
        .await
}

pub async fn insert_section(
    db: &mut Db,
    slug: &str,
    title: &str,
    media_kind: &str,
    nested: bool,
) -> toasty::Result<u64> {
    let next = next_section_position(db).await?;
    let s = toasty::create!(Section {
        slug: slug.to_string(),
        title: title.to_string(),
        media_kind: media_kind.to_string(),
        nested,
        position: next,
    })
    .exec(db)
    .await?;
    Ok(s.id)
}

async fn next_section_position(db: &mut Db) -> toasty::Result<i64> {
    let last = Section::all()
        .order_by(Section::fields().position().desc())
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
    let mut s = Section::get_by_id(db, &(id as u64)).await?;
    s.update()
        .title(title.to_string())
        .media_kind(media_kind.to_string())
        .nested(nested)
        .exec(db)
        .await
}

pub async fn update_section_title(db: &mut Db, id: i64, title: &str) -> toasty::Result<()> {
    let mut s = Section::get_by_id(db, &(id as u64)).await?;
    s.update().title(title.to_string()).exec(db).await
}

pub async fn delete_section(db: &mut Db, id: i64) -> toasty::Result<()> {
    let s = Section::get_by_id(db, &(id as u64)).await?;
    s.delete().exec(db).await
}

// ─── Collection queries ───────────────────────────────────────────────────

pub async fn fetch_collections(
    db: &mut Db,
    section_slug: &str,
    offset: usize,
    size: usize,
    search: Option<&str>,
) -> toasty::Result<Vec<Collection>> {
    let mut q = Collection::filter(
        Collection::fields()
            .section_slug()
            .eq(section_slug.to_string()),
    );
    if let Some(s) = search {
        q = q.filter(Collection::fields().title().like(format!("%{s}%")));
    }
    q.order_by((
        Collection::fields().position().asc(),
        Collection::fields().title().asc(),
        Collection::fields().id().asc(),
    ))
    .limit(size)
    .offset(offset)
    .exec(db)
    .await
}

pub async fn fetch_collections_count(
    db: &mut Db,
    section_slug: &str,
    search: Option<&str>,
) -> toasty::Result<i64> {
    let mut q = Collection::filter(
        Collection::fields()
            .section_slug()
            .eq(section_slug.to_string()),
    );
    if let Some(s) = search {
        q = q.filter(Collection::fields().title().like(format!("%{s}%")));
    }
    Ok(q.count().exec(db).await? as i64)
}

pub async fn fetch_collection_detail(
    db: &mut Db,
    section_slug: &str,
    collection_id: i64,
) -> toasty::Result<Option<Collection>> {
    Collection::filter(
        Collection::fields()
            .section_slug()
            .eq(section_slug.to_string())
            .and(Collection::fields().id().eq(collection_id as u64)),
    )
    .first()
    .exec(db)
    .await
}

pub async fn insert_empty_collection(db: &mut Db, section_slug: &str) -> toasty::Result<u64> {
    let section = Section::filter(Section::fields().slug().eq(section_slug.to_string()))
        .first()
        .exec(db)
        .await?
        .ok_or_else(|| err_not_found("section"))?;

    let next = next_collection_position(db, section.id).await?;
    let c = toasty::create!(Collection {
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
    Ok(c.id)
}

async fn next_collection_position(db: &mut Db, section_id: u64) -> toasty::Result<i64> {
    let last = Collection::filter(Collection::fields().section_id().eq(section_id))
        .order_by(Collection::fields().position().desc())
        .limit(1)
        .exec(db)
        .await?;
    Ok(last.first().map(|c| c.position + 1).unwrap_or(0))
}

pub enum CollectionField {
    Title,
    Description,
    Poster,
}

pub async fn update_collection_field(
    db: &mut Db,
    id: i64,
    field: CollectionField,
    value: Option<&str>,
) -> toasty::Result<()> {
    let mut c = Collection::get_by_id(db, &(id as u64)).await?;
    match field {
        // `title` is NOT NULL, so the update setter wants `String`, not
        // `Option<String>`. The server fn validates before calling.
        CollectionField::Title => {
            c.update()
                .title(value.unwrap_or_default().to_string())
                .exec(db)
                .await
        }
        // `description` and `poster` are nullable; `Option<String>` is
        // accepted directly, and `None` assigns SQL NULL.
        CollectionField::Description => {
            c.update()
                .description(value.map(String::from))
                .exec(db)
                .await
        }
        CollectionField::Poster => c.update().poster(value.map(String::from)).exec(db).await,
    }
}

pub async fn delete_collection(db: &mut Db, id: i64) -> toasty::Result<()> {
    let c = Collection::get_by_id(db, &(id as u64)).await?;
    c.delete().exec(db).await
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
        Section::filter(Section::fields().slug().eq(slug.to_string()))
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

    let collection = Collection::get_by_id(&mut tx, &(id as u64)).await?;
    let poster_url = collection.poster.clone();

    let items = Item::filter(Item::fields().collection_id().eq(id as u64))
        .exec(&mut tx)
        .await?;
    let file_ids: Vec<u64> = items.iter().map(|i| i.file_id).collect();

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
    let mut collection = Collection::get_by_id(&mut tx, &collection_id).await?;
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
