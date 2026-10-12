use serde::{Deserialize, Serialize};

use crate::app::constants::routes;

// ─── Tag canonicalization ─────────────────────────────────────────────────

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

// ─── Tag ──────────────────────────────────────────────────────────────────
//
// `Tag` has no `Vec` field, so it is both the wire type and the toasty model
// — no row/wire split needed.

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(toasty::Model))]
#[cfg_attr(feature = "ssr", table = "tags")]
pub struct Tag {
    #[cfg_attr(feature = "ssr", key)]
    #[cfg_attr(feature = "ssr", auto)]
    pub id: u64,
    #[cfg_attr(feature = "ssr", unique)]
    pub name: String,
}

// ─── Shared models (no `Vec` fields → single struct) ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(toasty::Model))]
#[cfg_attr(feature = "ssr", table = "files")]
pub struct File {
    #[cfg_attr(feature = "ssr", key)]
    #[cfg_attr(feature = "ssr", auto)]
    pub id: u64,
    #[cfg_attr(feature = "ssr", unique)]
    pub relative_path: String,
    pub size_bytes: i64,
    pub duration_secs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(toasty::Model))]
#[cfg_attr(feature = "ssr", table = "items")]
pub struct Item {
    #[cfg_attr(feature = "ssr", key)]
    #[cfg_attr(feature = "ssr", auto)]
    pub id: u64,
    #[cfg_attr(feature = "ssr", index)]
    pub collection_id: u64,
    pub number: i64,
    pub season_number: Option<i64>,
    pub title: Option<String>,
    pub poster: Option<String>,
    pub description: Option<String>,
    #[cfg_attr(feature = "ssr", index)]
    pub file_id: u64,
}

// ─── Row types (Toasty models for tables whose wire type carries tags) ────

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Serialize, Deserialize, toasty::Model)]
#[table = "sections"]
pub struct SectionRow {
    #[key]
    #[auto]
    pub id: u64,
    #[unique]
    pub slug: String,
    pub title: String,
    pub media_kind: String,
    pub nested: bool,
    pub position: i64,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Serialize, Deserialize, toasty::Model)]
#[table = "collections"]
pub struct CollectionRow {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub section_id: u64,
    #[index]
    pub section_slug: String,
    pub title: String,
    pub poster: Option<String>,
    pub description: Option<String>,
    pub position: i64,
    pub items_count: i64,
}

// ─── Join tables ──────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Serialize, Deserialize, toasty::Model)]
#[table = "section_tags"]
pub struct SectionTag {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub section_id: u64,
    #[index]
    pub tag_id: u64,
}

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Serialize, Deserialize, toasty::Model)]
#[table = "collection_tags"]
pub struct CollectionTag {
    #[key]
    #[auto]
    pub id: u64,
    #[index]
    pub collection_id: u64,
    #[index]
    pub tag_id: u64,
}

// ─── Domain models (wire types) ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: u64,
    pub slug: String,
    pub title: String,
    pub media_kind: String,
    pub nested: bool,
    pub position: i64,
    #[serde(default)]
    pub tags: Vec<Tag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: u64,
    pub section_id: u64,
    pub section_slug: String,
    pub title: String,
    pub poster: Option<String>,
    pub description: Option<String>,
    pub position: i64,
    pub items_count: i64,
    #[serde(default)]
    pub tags: Vec<Tag>,
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
impl From<CollectionRow> for Collection {
    fn from(row: CollectionRow) -> Self {
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
