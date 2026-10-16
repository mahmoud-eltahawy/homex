use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::constants::routes;
use crate::app::ids::Slug;
use crate::app::types::{FileExt, PosterUrl, TagName};

// ─── Tag canonicalization ─────────────────────────────────────────────────

/// Sort and deduplicate tag names.
///
/// Both operations use [`TagName`]'s case-insensitive `Ord` and `Eq`, so
/// `["Action", "ACTION"]` collapses to one entry. Trimming and empty-string
/// rejection already happened at construction, so there is nothing left to
/// normalize here.
pub fn canonicalize_tags(tags: impl IntoIterator<Item = TagName>) -> Vec<TagName> {
    let mut v: Vec<TagName> = tags.into_iter().collect();
    v.sort();
    v.dedup();
    v
}

// ─── SearchQuery ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchQuery {
    pub tags: Vec<TagName>,
    pub text: Option<String>,
}

impl SearchQuery {
    pub fn parse(raw: &str) -> Self {
        let mut tags: Vec<TagName> = Vec::new();
        let mut words: Vec<&str> = Vec::new();

        for word in raw.split_whitespace() {
            match word.strip_prefix('#').and_then(|t| TagName::new(t).ok()) {
                Some(name) => tags.push(name),
                None => words.push(word),
            }
        }

        tags.sort();
        tags.dedup();

        let text = if words.is_empty() {
            None
        } else {
            Some(words.join(" "))
        };
        SearchQuery { tags, text }
    }

    pub fn is_empty(&self) -> bool {
        self.tags.is_empty() && self.text.is_none()
    }
}

// ─── MediaKind ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Video,
    Audio,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidMediaKind;

impl std::fmt::Display for InvalidMediaKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("media_kind must be 'video' or 'audio'")
    }
}

impl std::error::Error for InvalidMediaKind {}

impl MediaKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Video => "video",
            Self::Audio => "audio",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Video => "Video",
            Self::Audio => "Audio",
        }
    }

    pub fn from_extension(ext: &FileExt) -> Option<Self> {
        const VIDEO_EXTS: &[&str] = &[
            "mp4", "mkv", "mov", "webm", "avi", "m4v", "wmv", "flv", "ts", "mpg", "mpeg", "3gp",
            "ogv",
        ];
        const AUDIO_EXTS: &[&str] = &[
            "mp3", "m4a", "aac", "wav", "ogg", "oga", "opus", "flac", "wma", "aiff", "aif", "alac",
            "ape",
        ];
        let ext = ext.as_str();
        if VIDEO_EXTS.iter().any(|e| *e == ext) {
            Some(Self::Video)
        } else if AUDIO_EXTS.iter().any(|e| *e == ext) {
            Some(Self::Audio)
        } else {
            None
        }
    }

    pub fn from_filename(name: &str) -> Option<Self> {
        let ext = FileExt::from_filename(name)?;
        Self::from_extension(&ext)
    }
}

impl TryFrom<&str> for MediaKind {
    type Error = InvalidMediaKind;
    fn try_from(v: &str) -> Result<Self, Self::Error> {
        match v.to_ascii_lowercase().as_str() {
            "video" => Ok(Self::Video),
            "audio" => Ok(Self::Audio),
            _ => Err(InvalidMediaKind),
        }
    }
}

// ─── CollectionField ──────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CollectionField {
    Title,
    Description,
    Poster,
}

// ─── CollectionKind (SSR only) ────────────────────────────────────────────

#[cfg(feature = "ssr")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectionKind {
    Video,
    Audio,
}

// ─── ModelError ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ModelError(String);

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ModelError {}

impl From<crate::app::ids::SlugError> for ModelError {
    fn from(e: crate::app::ids::SlugError) -> Self {
        ModelError(format!("invalid slug: {e}"))
    }
}

impl From<InvalidMediaKind> for ModelError {
    fn from(e: InvalidMediaKind) -> Self {
        ModelError(format!("invalid media_kind: {e}"))
    }
}

#[cfg(feature = "ssr")]
impl From<ModelError> for sqlx::Error {
    fn from(e: ModelError) -> Self {
        sqlx::Error::Decode(Box::new(e))
    }
}

// ─── Tag ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: Uuid,
    pub name: TagName,
}

/// The raw shape of a row from the `tags` table.
///
/// Kept separate from [`Tag`] because `TagName` has a validating constructor
/// and no `sqlx::Decode` impl. Every `sqlx::query_as!` that selects a tag
/// targets this struct, and the `From` impl below runs the value through
/// `TagName::from_stored` — the trusting path, since every name written to
/// the DB passed through `TagName::new`.
#[cfg(feature = "ssr")]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TagRow {
    pub id: Uuid,
    pub name: String,
}

#[cfg(feature = "ssr")]
impl From<TagRow> for Tag {
    fn from(row: TagRow) -> Self {
        Self {
            id: row.id,
            name: TagName::from_stored(row.name),
        }
    }
}

// ─── Item (wire + row) ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(sqlx::FromRow))]
pub struct Item {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub number: i64,
    pub season_number: Option<i64>,
    pub title: Option<String>,
    pub poster: Option<String>,
    pub description: Option<String>,
    pub file_id: Uuid,
}

// ─── Row types (SSR-only) ─────────────────────────────────────────────────

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SectionRow {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub media_kind: String,
    pub nested: bool,
    pub position: i64,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct VideoCollectionRow {
    pub id: Uuid,
    pub section_id: Uuid,
    pub section_slug: String,
    pub title: String,
    pub poster: String,
    pub description: Option<String>,
    pub position: i64,
    pub items_count: i64,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AudioCollectionRow {
    pub id: Uuid,
    pub section_id: Uuid,
    pub section_slug: String,
    pub title: String,
    pub poster: Option<String>,
    pub description: Option<String>,
    pub position: i64,
    pub items_count: i64,
}

// ─── Domain models (wire types) ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: Uuid,
    pub slug: Slug,
    pub title: String,
    pub media_kind: MediaKind,
    pub nested: bool,
    pub position: i64,
    #[serde(default)]
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: Uuid,
    pub section_id: Uuid,
    pub section_slug: Slug,
    pub title: String,
    pub poster: Option<PosterUrl>,
    pub description: Option<String>,
    pub position: i64,
    pub items_count: i64,
    #[serde(default)]
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CollectionSummary {
    pub id: Uuid,
    pub title: String,
    pub section_slug: Slug,
    pub section_title: String,
    pub nested: bool,
}

// ─── Row → domain conversions ─────────────────────────────────────────────

#[cfg(feature = "ssr")]
impl TryFrom<SectionRow> for Section {
    type Error = ModelError;
    fn try_from(row: SectionRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            slug: Slug::from_stored(row.slug),
            title: row.title,
            media_kind: MediaKind::try_from(row.media_kind.as_str())?,
            nested: row.nested,
            position: row.position,
            tags: Vec::new(),
        })
    }
}

#[cfg(feature = "ssr")]
impl From<VideoCollectionRow> for Collection {
    fn from(row: VideoCollectionRow) -> Self {
        Self {
            id: row.id,
            section_id: row.section_id,
            section_slug: Slug::from_stored(row.section_slug),
            title: row.title,
            poster: Some(PosterUrl::from_stored(row.poster)),
            description: row.description,
            position: row.position,
            items_count: row.items_count,
            tags: Vec::new(),
        }
    }
}

#[cfg(feature = "ssr")]
impl From<AudioCollectionRow> for Collection {
    fn from(row: AudioCollectionRow) -> Self {
        Self {
            id: row.id,
            section_id: row.section_id,
            section_slug: Slug::from_stored(row.section_slug),
            title: row.title,
            poster: row.poster.map(PosterUrl::from_stored),
            description: row.description,
            position: row.position,
            items_count: row.items_count,
            tags: Vec::new(),
        }
    }
}

// ─── Domain methods ───────────────────────────────────────────────────────

impl Section {
    pub fn href(&self) -> String {
        routes::section(self.slug.as_str())
    }
    pub fn new_label(&self) -> &'static str {
        match (self.media_kind, self.nested) {
            (MediaKind::Video, false) => "Add new video",
            (MediaKind::Video, true) => "Add new series",
            (MediaKind::Audio, false) => "Add new audio track",
            (MediaKind::Audio, true) => "Add new audio group",
        }
    }
    pub fn badge_label(&self) -> &'static str {
        match (self.media_kind, self.nested) {
            (MediaKind::Video, false) => "Video",
            (MediaKind::Video, true) => "Series",
            (MediaKind::Audio, false) => "Audio track",
            (MediaKind::Audio, true) => "Audio group",
        }
    }
}

impl Collection {
    pub fn href(&self) -> String {
        routes::collection(self.section_slug.as_str(), self.id)
    }
}

impl Item {
    pub fn display_title(&self) -> String {
        self.title
            .clone()
            .unwrap_or_else(|| format!("Item {}", self.number + 1))
    }
    pub fn file_path(&self) -> String {
        routes::media(self.file_id)
    }
    pub fn href(&self, section_slug: &str) -> String {
        routes::item(section_slug, self.collection_id, self.id)
    }
}
