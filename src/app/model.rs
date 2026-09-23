use serde::{Deserialize, Serialize};

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

// ─── Models ───────────────────────────────────────────────────────────────

#[cfg(feature = "ssr")]
#[derive(Debug, Clone, Serialize, Deserialize, toasty::Model)]
#[table = "files"]
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
#[cfg_attr(feature = "ssr", table = "sections")]
pub struct Section {
    #[cfg_attr(feature = "ssr", key)]
    #[cfg_attr(feature = "ssr", auto)]
    pub id: u64,
    #[cfg_attr(feature = "ssr", unique)]
    pub slug: String,
    pub title: String,
    pub media_kind: String,
    pub nested: bool,
    pub position: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(toasty::Model))]
#[cfg_attr(feature = "ssr", table = "collections")]
pub struct Collection {
    #[cfg_attr(feature = "ssr", key)]
    #[cfg_attr(feature = "ssr", auto)]
    pub id: u64,
    #[cfg_attr(feature = "ssr", index)]
    pub section_id: u64,
    #[cfg_attr(feature = "ssr", index)]
    pub section_slug: String,
    pub title: String,
    pub poster: Option<String>,
    pub description: Option<String>,
    pub position: i64,
    /// Denormalized counter — kept in sync by the insert/delete paths.
    /// The original SQLx schema computed this with a correlated subquery;
    /// Toasty has no equivalent expression, so we materialise it.
    pub items_count: i64,
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

// ─── Domain methods ───────────────────────────────────────────────────────

impl Section {
    pub fn media_kind(&self) -> MediaKind {
        MediaKind::try_from(self.media_kind.as_str()).unwrap_or(MediaKind::Video)
    }
    pub fn href(&self) -> String {
        format!("/s/{}", self.slug)
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
        format!("/s/{}/{}", self.section_slug, self.id)
    }
}

impl Item {
    /// Was `Item.file.path` in the SQLx model, where `file` was a nested
    /// `MediaFile` struct. Now it is derived from `file_id`.
    pub fn file_path(&self) -> String {
        format!("/media/{}", self.file_id)
    }
    pub fn href(&self, section_slug: &str) -> String {
        format!(
            "/s/{}/{}/item/{}",
            section_slug, self.collection_id, self.id
        )
    }

    pub fn display_title(&self) -> String {
        self.title
            .clone()
            .unwrap_or_else(|| format!("Item {}", self.number + 1))
    }
}
