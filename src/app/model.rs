use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MediaFile {
    pub id: u64,
    pub path: String,
    pub size: u64,
    pub duration: u64,
}
// impl MediaFile unchanged

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaKind {
    Video,
    Audio,
}

impl MediaKind {
    pub fn as_str(self) -> &'static str {
        match self { Self::Video => "video", Self::Audio => "audio" }
    }
    pub fn label(self) -> &'static str {
        match self { Self::Video => "فيديو", Self::Audio => "صوت" }
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Section {
    pub id: u64,
    pub slug: String,
    pub title: String,
    pub media_kind: MediaKind,
    pub nested: bool,
    pub position: i64,
    /// how many cards (collections) it holds — populated by the listing fn
    pub collections_count: u32,
}

impl Section {
    pub fn href(&self) -> String { format!("/s/{}", self.slug) }
    pub fn detail_href(&self, collection_id: u64) -> String {
        format!("/s/{}/{}", self.slug, collection_id)
    }
    /// label for the "create new" button inside the section page
    pub fn new_label(&self) -> &'static str {
        match (self.media_kind, self.nested) {
            (MediaKind::Video, false) => "إضافة فيديو جديد",
            (MediaKind::Video, true)  => "إضافة مسلسل جديد",
            (MediaKind::Audio, false) => "إضافة مقطع صوتي جديد",
            (MediaKind::Audio, true)  => "إضافة مجموعة صوتية جديدة",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Collection {
    pub id: u64,
    pub section_id: u64,
    pub section_slug: String,
    pub title: String,
    pub poster: Option<String>,
    pub description: Option<String>,
    pub items_count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: u64,
    pub collection_id: u64,
    pub number: i64,
    pub season_number: Option<i64>,
    pub title: Option<String>,
    pub poster: Option<String>,
    pub description: Option<String>,
    pub file: MediaFile,
}

impl Item {
    pub fn display_title(&self) -> String {
        self.title.clone().unwrap_or_else(|| format!("المقطع {}", self.number + 1))
    }
    pub fn href(&self, section_slug: &str) -> String {
        format!(
            "/s/{}/{}/item/{}",
            section_slug, self.collection_id, self.id
        )
    }
}
