use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::constants::routes;

// ─── Tag canonicalization (SSR only) ──────────────────────────────────────

#[cfg(feature = "ssr")]
pub fn canonicalize_tags<I, S>(tags: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut v: Vec<String> = tags
        .into_iter()
        .map(|t| t.as_ref().trim().to_string())
        .filter(|t| !t.is_empty())
        .collect();
    v.sort();
    v.dedup();
    v
}

// ─── Search parsing (SSR only) ────────────────────────────────────────────

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchQuery {
    pub tags: Vec<String>,
    pub text: Option<String>,
}

#[cfg(feature = "ssr")]
pub fn parse_search(raw: &str) -> SearchQuery {
    let mut tags: Vec<String> = Vec::new();
    let mut words: Vec<&str> = Vec::new();

    for word in raw.split_whitespace() {
        match word.strip_prefix('#') {
            Some(tag) if !tag.is_empty() => {
                tags.push(tag.to_string());
            }
            _ => words.push(word),
        }
    }

    tags.sort();
    tags.dedup_by(|a, b| a.eq_ignore_ascii_case(b));

    let text = if words.is_empty() {
        None
    } else {
        Some(words.join(" "))
    };

    SearchQuery { tags, text }
}

// ─── MediaKind ────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaKind {
    Video,
    Audio,
}

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

    pub fn from_extension(ext: &str) -> Option<Self> {
        const VIDEO_EXTS: &[&str] = &[
            "mp4", "mkv", "mov", "webm", "avi", "m4v", "wmv", "flv", "ts", "mpg", "mpeg", "3gp",
            "ogv",
        ];
        const AUDIO_EXTS: &[&str] = &[
            "mp3", "m4a", "aac", "wav", "ogg", "oga", "opus", "flac", "wma", "aiff", "aif", "alac",
            "ape",
        ];
        let ext = ext.to_lowercase();
        if VIDEO_EXTS.iter().any(|e| *e == ext) {
            Some(Self::Video)
        } else if AUDIO_EXTS.iter().any(|e| *e == ext) {
            Some(Self::Audio)
        } else {
            None
        }
    }

    pub fn from_filename(name: &str) -> Option<Self> {
        let ext = name.rsplit_once('.').map(|(_, e)| e)?;
        Self::from_extension(ext)
    }
}

impl TryFrom<&str> for MediaKind {
    type Error = &'static str;
    fn try_from(v: &str) -> Result<Self, Self::Error> {
        match v.to_ascii_lowercase().as_str() {
            "video" => Ok(Self::Video),
            "audio" => Ok(Self::Audio),
            _ => Err("media_kind must be 'video' or 'audio'"),
        }
    }
}

// ─── CollectionKind (SSR only) ────────────────────────────────────────────
//
// Internal to the db dispatcher. Never crosses the wire.

#[cfg(feature = "ssr")]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectionKind {
    Video,
    Audio,
}

// ─── Tag ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(toasty::Model))]
#[cfg_attr(feature = "ssr", table = "tags")]
pub struct Tag {
    #[cfg_attr(feature = "ssr", key)]
    #[cfg_attr(feature = "ssr", auto)]
    pub id: Uuid,
    #[cfg_attr(feature = "ssr", unique)]
    pub name: String,
}

// ─── File (SSR only) ──────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Serialize, Deserialize, toasty::Model)]
#[table = "files"]
pub struct File {
    #[key]
    #[auto]
    pub id: Uuid,
    #[unique]
    pub relative_path: String,
    pub size_bytes: i64,
    pub duration_secs: i64,
}

// ─── Item (wire + toasty model) ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(toasty::Model))]
#[cfg_attr(feature = "ssr", table = "items")]
pub struct Item {
    #[cfg_attr(feature = "ssr", key)]
    #[cfg_attr(feature = "ssr", auto)]
    pub id: Uuid,
    #[cfg_attr(feature = "ssr", index)]
    pub collection_id: Uuid,
    pub number: i64,
    pub season_number: Option<i64>,
    pub title: Option<String>,
    pub poster: Option<String>,
    pub description: Option<String>,
    #[cfg_attr(feature = "ssr", index)]
    pub file_id: Uuid,
}

// ─── Row types (SSR-only Toasty models) ───────────────────────────────────

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Serialize, Deserialize, toasty::Model)]
#[table = "sections"]
pub struct SectionRow {
    #[key]
    #[auto]
    pub id: Uuid,
    #[unique]
    pub slug: String,
    pub title: String,
    pub media_kind: String,
    pub nested: bool,
    pub position: i64,
}

/// A collection belonging to a video section.
///
/// A row can only exist because a video was uploaded into it and a poster
/// frame was extracted from that video. `poster` is therefore `String`, not
/// `Option<String>` — the invariant is enforced by the server function that
/// inserts the row (`extract_and_insert_new_collection`).
#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Serialize, Deserialize, toasty::Model)]
#[table = "video_collections"]
pub struct VideoCollectionRow {
    #[key]
    pub id: Uuid,
    #[index]
    pub section_id: Uuid,
    #[index]
    pub section_slug: String,
    pub title: String,
    pub poster: String,
    pub description: Option<String>,
    pub position: i64,
    pub items_count: i64,
}

/// A collection belonging to an audio section.
///
/// Audio has no frame to extract, so the poster is user-supplied and
/// genuinely optional.
#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Serialize, Deserialize, toasty::Model)]
#[table = "audio_collections"]
pub struct AudioCollectionRow {
    #[key]
    pub id: Uuid,
    #[index]
    pub section_id: Uuid,
    #[index]
    pub section_slug: String,
    pub title: String,
    pub poster: Option<String>,
    pub description: Option<String>,
    pub position: i64,
    pub items_count: i64,
}

// ─── Join tables (SSR-only) ───────────────────────────────────────────────

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Serialize, Deserialize, toasty::Model)]
#[table = "section_tags"]
pub struct SectionTag {
    #[key]
    #[auto]
    pub id: Uuid,
    #[index]
    pub section_id: Uuid,
    #[index]
    pub tag_id: Uuid,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Serialize, Deserialize, toasty::Model)]
#[table = "collection_tags"]
pub struct CollectionTag {
    #[key]
    #[auto]
    pub id: Uuid,
    #[index]
    pub collection_id: Uuid,
    #[index]
    pub tag_id: Uuid,
}

// ─── Domain models (wire types) ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub media_kind: String,
    pub nested: bool,
    pub position: i64,
    #[serde(default)]
    pub tags: Vec<Tag>,
}

/// Read model for grids, cards, listings. The DB stores video and audio
/// collections in separate tables; this type lets the shared UI treat them
/// uniformly. `poster` is `Option` because audio collections may never get
/// one — for video collections it is always `Some` in practice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: Uuid,
    pub section_id: Uuid,
    pub section_slug: String,
    pub title: String,
    pub poster: Option<String>,
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
    pub section_slug: String,
    pub section_title: String,
    pub nested: bool,
}

// ─── Row → domain conversions ─────────────────────────────────────────────

#[cfg(feature = "ssr")]
impl From<SectionRow> for Section {
    fn from(row: SectionRow) -> Self {
        Self {
            id: row.id,
            slug: row.slug,
            title: row.title,
            media_kind: row.media_kind,
            nested: row.nested,
            position: row.position,
            tags: Vec::new(),
        }
    }
}

#[cfg(feature = "ssr")]
impl From<VideoCollectionRow> for Collection {
    fn from(row: VideoCollectionRow) -> Self {
        Self {
            id: row.id,
            section_id: row.section_id,
            section_slug: row.section_slug,
            title: row.title,
            poster: Some(row.poster),
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
            section_slug: row.section_slug,
            title: row.title,
            poster: row.poster,
            description: row.description,
            position: row.position,
            items_count: row.items_count,
            tags: Vec::new(),
        }
    }
}

// ─── Domain methods ───────────────────────────────────────────────────────

impl Section {
    pub fn media_kind(&self) -> MediaKind {
        MediaKind::try_from(self.media_kind.as_str()).unwrap_or(MediaKind::Video)
    }
    pub fn href(&self) -> String {
        routes::section(&self.slug)
    }
    pub fn new_label(&self) -> &'static str {
        match (self.media_kind(), self.nested) {
            (MediaKind::Video, false) => "Add new video",
            (MediaKind::Video, true) => "Add new series",
            (MediaKind::Audio, false) => "Add new audio track",
            (MediaKind::Audio, true) => "Add new audio group",
        }
    }
    pub fn badge_label(&self) -> &'static str {
        match (self.media_kind(), self.nested) {
            (MediaKind::Video, false) => "Video",
            (MediaKind::Video, true) => "Series",
            (MediaKind::Audio, false) => "Audio track",
            (MediaKind::Audio, true) => "Audio group",
        }
    }
}

impl Collection {
    pub fn href(&self) -> String {
        routes::collection(&self.section_slug, self.id)
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
