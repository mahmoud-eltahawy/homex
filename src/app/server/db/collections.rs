use toasty::Db;
use uuid::Uuid;

use super::err_not_found;
use super::files::purge_orphan_files;
use super::tags::{collection_ids_with_all_tags, fetch_collection_tags};
use crate::app::model::{
    Collection, CollectionRow, CollectionSummary, CollectionTag, Item, SectionRow, parse_search,
};

// ─── Collection queries ───────────────────────────────────────────────────

pub async fn fetch_collections(
    db: &mut Db,
    section_slug: &str,
    offset: usize,
    size: usize,
    search: Option<&str>,
) -> toasty::Result<Vec<Collection>> {
    let parsed = search.map(parse_search).unwrap_or_default();

    let tag_ids: Option<Vec<Uuid>> = if parsed.tags.is_empty() {
        None
    } else {
        Some(collection_ids_with_all_tags(db, &parsed.tags).await?)
    };
    if tag_ids.as_ref().is_some_and(Vec::is_empty) {
        return Ok(Vec::new());
    }

    let rows = if let Some(ids) = &tag_ids {
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

    let tag_ids: Option<Vec<Uuid>> = if parsed.tags.is_empty() {
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
    collection_id: Uuid,
) -> toasty::Result<Option<Collection>> {
    let Some(row) = CollectionRow::filter(
        CollectionRow::fields()
            .section_slug()
            .eq(section_slug.to_string())
            .and(CollectionRow::fields().id().eq(collection_id)),
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

pub async fn insert_empty_collection(db: &mut Db, section_slug: &str) -> toasty::Result<Uuid> {
    let section = SectionRow::filter(SectionRow::fields().slug().eq(section_slug.to_string()))
        .first()
        .exec(db)
        .await?
        .ok_or_else(|| err_not_found("section"))?;

    let next = next_position(db, section.id).await?;
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

async fn next_position(db: &mut Db, section_id: Uuid) -> toasty::Result<i64> {
    let last = CollectionRow::filter(CollectionRow::fields().section_id().eq(section_id))
        .order_by(CollectionRow::fields().position().desc())
        .limit(1)
        .exec(db)
        .await?;
    Ok(last.first().map(|c| c.position + 1).unwrap_or(0))
}

// ─── Field update ─────────────────────────────────────────────────────────

pub enum CollectionField {
    Title,
    Description,
    Poster,
}

pub async fn update_collection_field(
    db: &mut Db,
    id: Uuid,
    field: CollectionField,
    value: Option<&str>,
) -> toasty::Result<()> {
    let mut row = CollectionRow::get_by_id(db, &id).await?;
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
    }
}

// ─── Move collection to another section ───────────────────────────────────

#[derive(Debug)]
pub enum MoveCollectionError {
    SectionNotFound,
    MediaKindMismatch,
    Db(toasty::Error),
}

impl From<toasty::Error> for MoveCollectionError {
    fn from(e: toasty::Error) -> Self {
        MoveCollectionError::Db(e)
    }
}

pub struct CollectionMove {
    pub from_slug: String,
    pub poster_url: Option<String>,
}

pub async fn move_collection_to_section(
    db: &mut Db,
    collection_id: Uuid,
    to_section_slug: &str,
) -> Result<CollectionMove, MoveCollectionError> {
    let to_section =
        SectionRow::filter(SectionRow::fields().slug().eq(to_section_slug.to_string()))
            .first()
            .exec(db)
            .await?
            .ok_or(MoveCollectionError::SectionNotFound)?;

    let mut c = CollectionRow::get_by_id(db, &collection_id).await?;

    let from_section = SectionRow::filter(SectionRow::fields().slug().eq(c.section_slug.clone()))
        .first()
        .exec(db)
        .await?;

    if let Some(from) = from_section
        && from.media_kind != to_section.media_kind
    {
        return Err(MoveCollectionError::MediaKindMismatch);
    }

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

// ─── Destination summaries ────────────────────────────────────────────────

pub async fn fetch_collection_summaries(
    db: &mut Db,
    media_kind: &str,
) -> toasty::Result<Vec<CollectionSummary>> {
    let sections = SectionRow::filter(SectionRow::fields().media_kind().eq(media_kind.to_string()))
        .order_by(SectionRow::fields().title().asc())
        .exec(db)
        .await?;

    let mut out: Vec<CollectionSummary> = Vec::new();
    for section in &sections {
        let rows = CollectionRow::filter(
            CollectionRow::fields()
                .section_slug()
                .eq(section.slug.clone()),
        )
        .order_by(CollectionRow::fields().title().asc())
        .exec(db)
        .await?;

        out.extend(rows.into_iter().map(|c| CollectionSummary {
            section_title: section.title.clone(),
            id: c.id,
            title: c.title,
            section_slug: c.section_slug,
            nested: section.nested, // <-- new
        }));
    }

    Ok(out)
}

// ─── Cascade deletion ─────────────────────────────────────────────────────

pub struct CollectionDeletion {
    pub media_paths: Vec<String>,
    pub poster_url: Option<String>,
}

pub async fn delete_collection_cascade(
    db: &mut Db,
    id: Uuid,
) -> toasty::Result<CollectionDeletion> {
    let mut tx = db.transaction().await?;

    let collection = CollectionRow::get_by_id(&mut tx, &id).await?;
    let poster_url = collection.poster.clone();

    let items = Item::filter(Item::fields().collection_id().eq(id))
        .exec(&mut tx)
        .await?;
    let file_ids: Vec<Uuid> = items.iter().map(|i| i.file_id).collect();

    for row in CollectionTag::filter(CollectionTag::fields().collection_id().eq(id))
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
