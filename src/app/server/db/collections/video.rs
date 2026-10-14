use toasty::Db;
use uuid::Uuid;

use super::super::err_not_found;
use super::super::files::purge_orphan_files;
use super::super::tags::{collection_ids_with_all_tags, fetch_collection_tags};
use super::{CollectionDeletion, CollectionField, CollectionMove, MoveCollectionError};
use crate::app::model::{
    Collection, CollectionSummary, CollectionTag, Item, MediaKind, SectionRow, VideoCollectionRow,
};

// ─── Read ─────────────────────────────────────────────────────────────────

pub(super) async fn fetch(
    db: &mut Db,
    section_slug: &str,
    offset: usize,
    size: usize,
    search: Option<&str>,
) -> toasty::Result<Vec<Collection>> {
    use crate::app::model::parse_search;

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
        let mut q = VideoCollectionRow::filter(
            VideoCollectionRow::fields()
                .section_slug()
                .eq(section_slug.to_string()),
        );
        if let Some(t) = &parsed.text {
            q = q.filter(VideoCollectionRow::fields().title().like(format!("%{t}%")));
        }
        q.order_by((
            VideoCollectionRow::fields().position().asc(),
            VideoCollectionRow::fields().title().asc(),
            VideoCollectionRow::fields().id().asc(),
        ))
        .exec(db)
        .await?
        .into_iter()
        .filter(|r| ids.contains(&r.id))
        .skip(offset)
        .take(size)
        .collect()
    } else {
        let mut q = VideoCollectionRow::filter(
            VideoCollectionRow::fields()
                .section_slug()
                .eq(section_slug.to_string()),
        );
        if let Some(t) = &parsed.text {
            q = q.filter(VideoCollectionRow::fields().title().like(format!("%{t}%")));
        }
        q.order_by((
            VideoCollectionRow::fields().position().asc(),
            VideoCollectionRow::fields().title().asc(),
            VideoCollectionRow::fields().id().asc(),
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

pub(super) async fn count(
    db: &mut Db,
    section_slug: &str,
    search: Option<&str>,
) -> toasty::Result<i64> {
    use crate::app::model::parse_search;

    let parsed = search.map(parse_search).unwrap_or_default();

    let tag_ids: Option<Vec<Uuid>> = if parsed.tags.is_empty() {
        None
    } else {
        Some(collection_ids_with_all_tags(db, &parsed.tags).await?)
    };
    if tag_ids.as_ref().is_some_and(Vec::is_empty) {
        return Ok(0);
    }

    let mut q = VideoCollectionRow::filter(
        VideoCollectionRow::fields()
            .section_slug()
            .eq(section_slug.to_string()),
    );
    if let Some(t) = &parsed.text {
        q = q.filter(VideoCollectionRow::fields().title().like(format!("%{t}%")));
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

pub(super) async fn detail(
    db: &mut Db,
    section_slug: &str,
    collection_id: Uuid,
) -> toasty::Result<Option<Collection>> {
    let Some(row) = VideoCollectionRow::filter(
        VideoCollectionRow::fields()
            .section_slug()
            .eq(section_slug.to_string())
            .and(VideoCollectionRow::fields().id().eq(collection_id)),
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

pub(super) async fn summaries(
    db: &mut Db,
    media_kind: &str,
) -> toasty::Result<Vec<CollectionSummary>> {
    let sections = SectionRow::filter(SectionRow::fields().media_kind().eq(media_kind.to_string()))
        .order_by(SectionRow::fields().title().asc())
        .exec(db)
        .await?;

    let mut out: Vec<CollectionSummary> = Vec::new();
    for section in &sections {
        let rows = VideoCollectionRow::filter(
            VideoCollectionRow::fields()
                .section_slug()
                .eq(section.slug.clone()),
        )
        .order_by(VideoCollectionRow::fields().title().asc())
        .exec(db)
        .await?;

        out.extend(rows.into_iter().map(|c| CollectionSummary {
            section_title: section.title.clone(),
            id: c.id,
            title: c.title,
            section_slug: c.section_slug,
            nested: section.nested,
        }));
    }
    Ok(out)
}

// ─── Insert ───────────────────────────────────────────────────────────────

async fn next_position(db: &mut toasty::Transaction<'_>, section_id: Uuid) -> toasty::Result<i64> {
    let last = VideoCollectionRow::filter(VideoCollectionRow::fields().section_id().eq(section_id))
        .order_by(VideoCollectionRow::fields().position().desc())
        .limit(1)
        .exec(db)
        .await?;
    Ok(last.first().map(|c| c.position + 1).unwrap_or(0))
}

async fn write_tags(
    tx: &mut toasty::Transaction<'_>,
    collection_id: Uuid,
    tags: &[String],
) -> toasty::Result<()> {
    use crate::app::model::{Tag, canonicalize_tags};

    for name in canonicalize_tags(tags) {
        let tag = match Tag::filter(Tag::fields().name().eq(name.clone()))
            .first()
            .exec(tx)
            .await?
        {
            Some(t) => t,
            None => toasty::create!(Tag { name: name.clone() }).exec(tx).await?,
        };
        toasty::create!(CollectionTag {
            collection_id,
            tag_id: tag.id,
        })
        .exec(tx)
        .await?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
pub(super) async fn insert(
    db: &mut Db,
    id: Uuid,
    section_slug: &str,
    title: &str,
    description: Option<&str>,
    tags: &[String],
    poster: &str,
) -> toasty::Result<()> {
    let mut tx = db.transaction().await?;

    let section = SectionRow::filter(SectionRow::fields().slug().eq(section_slug.to_string()))
        .first()
        .exec(&mut tx)
        .await?
        .ok_or_else(|| err_not_found("section"))?;

    let next = next_position(&mut tx, section.id).await?;

    toasty::create!(VideoCollectionRow {
        id,
        section_id: section.id,
        section_slug: section.slug.clone(),
        title: title.to_string(),
        poster: poster.to_string(),
        description: description.map(String::from),
        position: next,
        items_count: 0,
    })
    .exec(&mut tx)
    .await?;

    write_tags(&mut tx, id, tags).await?;

    tx.commit().await?;
    Ok(())
}

// ─── Field / poster updates ───────────────────────────────────────────────

pub(super) async fn update_field(
    db: &mut Db,
    id: Uuid,
    field: CollectionField,
    value: Option<&str>,
) -> toasty::Result<()> {
    let mut row = VideoCollectionRow::get_by_id(db, &id).await?;
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
        // Unreachable: video posters are set only via `set_poster`.
        CollectionField::Poster => Ok(()),
    }
}

pub(super) async fn set_poster(db: &mut Db, id: Uuid, url: &str) -> toasty::Result<()> {
    let mut row = VideoCollectionRow::get_by_id(db, &id).await?;
    row.update().poster(url.to_string()).exec(db).await
}

// ─── Delete ───────────────────────────────────────────────────────────────

pub(super) async fn delete_cascade(db: &mut Db, id: Uuid) -> toasty::Result<CollectionDeletion> {
    let mut tx = db.transaction().await?;

    let collection = VideoCollectionRow::get_by_id(&mut tx, &id).await?;
    let poster_url = Some(collection.poster.clone());

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

// ─── Move to another section ──────────────────────────────────────────────

pub(super) async fn move_to_section(
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

    if to_section.media_kind != MediaKind::Video.as_str() {
        return Err(MoveCollectionError::MediaKindMismatch);
    }

    let mut c = VideoCollectionRow::get_by_id(db, &collection_id).await?;
    let from_slug = c.section_slug.clone();
    let poster_url = Some(c.poster.clone());

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
