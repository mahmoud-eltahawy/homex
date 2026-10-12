use toasty::Db;

use super::tags::fetch_section_tags;
use crate::app::model::{CollectionRow, Section, SectionRow, SectionTag};

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
    let next = next_position(db).await?;
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

async fn next_position(db: &mut Db) -> toasty::Result<i64> {
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

// ─── Section deletion ─────────────────────────────────────────────────────

/// Outcome of a delete attempt. The section is only removed if it has no
/// dependent collections — otherwise the caller decides how to surface
/// the refusal.
pub enum DeleteSectionOutcome {
    Deleted,
    NotEmpty { collection_count: i64 },
}

pub async fn delete_section(db: &mut Db, id: i64) -> toasty::Result<DeleteSectionOutcome> {
    let section = SectionRow::get_by_id(db, &(id as u64)).await?;

    // Refuse to orphan collections. The check runs inside the same call so
    // no caller can accidentally skip it.
    let collection_count = CollectionRow::filter(
        CollectionRow::fields()
            .section_slug()
            .eq(section.slug.clone()),
    )
    .count()
    .exec(db)
    .await? as i64;

    if collection_count > 0 {
        return Ok(DeleteSectionOutcome::NotEmpty { collection_count });
    }

    // Drop the section↔tag joins. Tag rows themselves are shared and stay.
    for row in SectionTag::filter(SectionTag::fields().section_id().eq(id as u64))
        .exec(db)
        .await?
    {
        row.delete().exec(db).await?;
    }

    section.delete().exec(db).await?;
    Ok(DeleteSectionOutcome::Deleted)
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
