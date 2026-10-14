mod audio;
mod video;

use toasty::Db;
use uuid::Uuid;

use super::err_not_found;
use crate::app::model::{Collection, CollectionKind, CollectionSummary, MediaKind, SectionRow};

// ─── Kind lookups ─────────────────────────────────────────────────────────

pub(crate) async fn kind_of(db: &mut Db, id: Uuid) -> toasty::Result<Option<CollectionKind>> {
    use crate::app::model::{AudioCollectionRow, VideoCollectionRow};

    if !VideoCollectionRow::filter(VideoCollectionRow::fields().id().eq(id))
        .limit(1)
        .exec(db)
        .await?
        .is_empty()
    {
        return Ok(Some(CollectionKind::Video));
    }
    if AudioCollectionRow::filter(AudioCollectionRow::fields().id().eq(id))
        .limit(1)
        .exec(db)
        .await?
        .is_empty()
    {
        return Ok(Some(CollectionKind::Audio));
    }
    Ok(None)
}

pub(crate) async fn kind_of_tx(
    tx: &mut toasty::Transaction<'_>,
    id: Uuid,
) -> toasty::Result<Option<CollectionKind>> {
    use crate::app::model::{AudioCollectionRow, VideoCollectionRow};

    if VideoCollectionRow::filter(VideoCollectionRow::fields().id().eq(id))
        .limit(1)
        .exec(tx)
        .await?
        .is_empty()
    {
        return Ok(Some(CollectionKind::Video));
    }
    if AudioCollectionRow::filter(AudioCollectionRow::fields().id().eq(id))
        .limit(1)
        .exec(tx)
        .await?
        .is_empty()
    {
        return Ok(Some(CollectionKind::Audio));
    }
    Ok(None)
}

pub(crate) async fn section_kind(db: &mut Db, section_slug: &str) -> toasty::Result<MediaKind> {
    let section = SectionRow::filter(SectionRow::fields().slug().eq(section_slug.to_string()))
        .first()
        .exec(db)
        .await?
        .ok_or_else(|| err_not_found("section"))?;
    MediaKind::try_from(section.media_kind.as_str())
        .map_err(|e| toasty::Error::from_args(format_args!("section has invalid media_kind: {e}")))
}

// ─── Read entry points ────────────────────────────────────────────────────

pub async fn fetch_collections(
    db: &mut Db,
    section_slug: &str,
    offset: usize,
    size: usize,
    search: Option<&str>,
) -> toasty::Result<Vec<Collection>> {
    match section_kind(db, section_slug).await? {
        MediaKind::Video => video::fetch(db, section_slug, offset, size, search).await,
        MediaKind::Audio => audio::fetch(db, section_slug, offset, size, search).await,
    }
}

pub async fn fetch_collections_count(
    db: &mut Db,
    section_slug: &str,
    search: Option<&str>,
) -> toasty::Result<i64> {
    match section_kind(db, section_slug).await? {
        MediaKind::Video => video::count(db, section_slug, search).await,
        MediaKind::Audio => audio::count(db, section_slug, search).await,
    }
}

pub async fn fetch_collection_detail(
    db: &mut Db,
    section_slug: &str,
    collection_id: Uuid,
) -> toasty::Result<Option<Collection>> {
    match section_kind(db, section_slug).await? {
        MediaKind::Video => video::detail(db, section_slug, collection_id).await,
        MediaKind::Audio => audio::detail(db, section_slug, collection_id).await,
    }
}

/// Look up a collection without knowing its section — used by the classify
/// path where only the collection id is available.
pub async fn fetch_collection_any(db: &mut Db, id: Uuid) -> toasty::Result<Option<Collection>> {
    use super::tags::fetch_collection_tags;
    use crate::app::model::{AudioCollectionRow, VideoCollectionRow};

    match kind_of(db, id).await? {
        None => Ok(None),
        Some(CollectionKind::Video) => {
            let Some(row) = VideoCollectionRow::filter(VideoCollectionRow::fields().id().eq(id))
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
        Some(CollectionKind::Audio) => {
            let Some(row) = AudioCollectionRow::filter(AudioCollectionRow::fields().id().eq(id))
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
    }
}

pub async fn fetch_collection_summaries(
    db: &mut Db,
    media_kind: &str,
) -> toasty::Result<Vec<CollectionSummary>> {
    match MediaKind::try_from(media_kind)
        .map_err(|e| toasty::Error::from_args(format_args!("bad media_kind: {e}")))?
    {
        MediaKind::Video => video::summaries(db, media_kind).await,
        MediaKind::Audio => audio::summaries(db, media_kind).await,
    }
}

// ─── Insert entry points ──────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
pub async fn insert_video_collection(
    db: &mut Db,
    id: Uuid,
    section_slug: &str,
    title: &str,
    description: Option<&str>,
    tags: &[String],
    poster: &str,
) -> toasty::Result<()> {
    video::insert(db, id, section_slug, title, description, tags, poster).await
}

#[allow(clippy::too_many_arguments)]
pub async fn insert_audio_collection(
    db: &mut Db,
    id: Uuid,
    section_slug: &str,
    title: &str,
    description: Option<&str>,
    tags: &[String],
    poster: Option<String>,
) -> toasty::Result<()> {
    audio::insert(db, id, section_slug, title, description, tags, poster).await
}

// ─── Field / poster updates ───────────────────────────────────────────────

pub async fn update_collection_field(
    db: &mut Db,
    id: Uuid,
    field: CollectionField,
    value: Option<&str>,
) -> toasty::Result<()> {
    match kind_of(db, id)
        .await?
        .ok_or_else(|| err_not_found("collection"))?
    {
        CollectionKind::Video => video::update_field(db, id, field, value).await,
        CollectionKind::Audio => audio::update_field(db, id, field, value).await,
    }
}

pub async fn set_collection_poster(db: &mut Db, id: Uuid, url: Option<&str>) -> toasty::Result<()> {
    match kind_of(db, id)
        .await?
        .ok_or_else(|| err_not_found("collection"))?
    {
        CollectionKind::Video => {
            let url = url.ok_or_else(|| {
                toasty::Error::from_args(format_args!("video collection poster cannot be cleared"))
            })?;
            video::set_poster(db, id, url).await
        }
        CollectionKind::Audio => audio::set_poster(db, id, url).await,
    }
}

// ─── Delete / move ────────────────────────────────────────────────────────

pub async fn delete_collection_cascade(
    db: &mut Db,
    id: Uuid,
) -> toasty::Result<CollectionDeletion> {
    match kind_of(db, id)
        .await?
        .ok_or_else(|| err_not_found("collection"))?
    {
        CollectionKind::Video => video::delete_cascade(db, id).await,
        CollectionKind::Audio => audio::delete_cascade(db, id).await,
    }
}

pub async fn move_collection_to_section(
    db: &mut Db,
    collection_id: Uuid,
    to_section_slug: &str,
) -> Result<CollectionMove, MoveCollectionError> {
    match kind_of(db, collection_id)
        .await
        .map_err(MoveCollectionError::Db)?
        .ok_or_else(|| MoveCollectionError::Db(err_not_found("collection")))?
    {
        CollectionKind::Video => video::move_to_section(db, collection_id, to_section_slug).await,
        CollectionKind::Audio => audio::move_to_section(db, collection_id, to_section_slug).await,
    }
}

// ─── Decrement-or-delete helper ───────────────────────────────────────────

pub(crate) async fn decrement_or_delete(
    tx: &mut toasty::Transaction<'_>,
    collection_id: Uuid,
    delta: i64,
) -> toasty::Result<Option<String>> {
    use crate::app::model::{AudioCollectionRow, CollectionTag, VideoCollectionRow};

    let Some(kind) = kind_of_tx(tx, collection_id).await? else {
        return Ok(None);
    };

    let poster_opt: Option<String> = match kind {
        CollectionKind::Video => {
            let mut c = VideoCollectionRow::get_by_id(tx, &collection_id).await?;
            let next = (c.items_count - delta).max(0);
            if next == 0 {
                let poster = c.poster.clone();
                for row in
                    CollectionTag::filter(CollectionTag::fields().collection_id().eq(collection_id))
                        .exec(tx)
                        .await?
                {
                    row.delete().exec(tx).await?;
                }
                c.delete().exec(tx).await?;
                Some(poster)
            } else {
                c.update().items_count(next).exec(tx).await?;
                None
            }
        }
        CollectionKind::Audio => {
            let mut c = AudioCollectionRow::get_by_id(tx, &collection_id).await?;
            let next = (c.items_count - delta).max(0);
            if next == 0 {
                let poster = c.poster.clone();
                for row in
                    CollectionTag::filter(CollectionTag::fields().collection_id().eq(collection_id))
                        .exec(tx)
                        .await?
                {
                    row.delete().exec(tx).await?;
                }
                c.delete().exec(tx).await?;
                poster
            } else {
                c.update().items_count(next).exec(tx).await?;
                None
            }
        }
    };
    Ok(poster_opt)
}

// ─── Shared shapes ────────────────────────────────────────────────────────

pub struct CollectionDeletion {
    pub media_paths: Vec<String>,
    pub poster_url: Option<String>,
}

pub struct CollectionMove {
    pub from_slug: String,
    pub poster_url: Option<String>,
}

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

pub enum CollectionField {
    Title,
    Description,
    Poster,
}
