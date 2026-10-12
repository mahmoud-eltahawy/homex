use toasty::Db;

use crate::app::model::{CollectionTag, SectionTag, Tag, canonicalize_tags};

// ─── Tag loaders / writers ────────────────────────────────────────────────

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
pub(crate) async fn collection_ids_with_all_tags(
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
