use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::app::view_schema::IdT;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MediaFile {
    pub id: u64,
    pub path: String,
    pub size: u64,
    pub duration: u64,
}

impl MediaFile {
    pub fn human_readable_size(&self) -> String {
        let bytes = self.size as f64;
        if bytes >= 1_000_000_000.0 {
            format!("{:.1} GB", bytes / 1_000_000_000.0)
        } else if bytes >= 1_000_000.0 {
            format!("{:.1} MG", bytes / 1_000_000.0)
        } else if bytes >= 1_000.0 {
            format!("{:.1} KB", bytes / 1_000.0)
        } else {
            format!("{} BYTE", bytes)
        }
    }
}

impl MediaFile {
    pub fn human_readable_duration(&self) -> String {
        let secs = self.duration;
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;
        if hours > 0 {
            format!("{} Hour And {} Minute", hours, minutes)
        } else if minutes > 0 {
            format!("{} Minute", minutes)
        } else {
            format!("{} Second", seconds)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Movie {
    pub id: u64,
    pub title: String,
    pub poster: Option<String>,
    pub description: Option<String>,
    pub file: MediaFile,
}

impl IdT for Movie {
    fn id(&self) -> u64 {
        self.id
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioGroup {
    pub id: u64,
    pub title: String,
    pub poster: Option<String>,
    pub description: Option<String>,
    pub audios_count: u32,
}

impl IdT for AudioGroup {
    fn id(&self) -> u64 {
        self.id
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum MediaType {
    Movie,
    Series,
    AudioGroup,
}

impl MediaType {
    pub fn listing_href(&self) -> String {
        match self {
            MediaType::Movie => "/movie",
            MediaType::Series => "/series",
            MediaType::AudioGroup => "/audio",
        }
        .to_string()
    }
    pub fn detail_href(&self, id: u64) -> String {
        format!("{}/detail/{}", self.listing_href(), id)
    }
}

impl Display for MediaType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MediaType::Movie => write!(f, "movie"),
            MediaType::Series => write!(f, "series"),
            MediaType::AudioGroup => write!(f, "audio"),
        }
    }
}

impl TryFrom<&str> for MediaType {
    type Error = &'static str;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_lowercase().as_str() {
            "movie" => Ok(MediaType::Movie),
            "series" => Ok(MediaType::Series),
            "audio" => Ok(MediaType::AudioGroup),
            _ => Err("Media type must be 'movie' or 'series'"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Episode {
    pub id: i64,
    pub season: u32,
    pub episode: u32,
    pub file: MediaFile,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Series {
    pub id: u64,
    pub title: String,
    pub poster: Option<String>,
    pub description: Option<String>,
    pub season_count: u32,
    pub season_summaries: Vec<SeasonSummary>,
}

impl IdT for Series {
    fn id(&self) -> u64 {
        self.id
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeasonSummary {
    pub season_number: u32,
    pub episode_count: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Season {
    pub season_number: u32,
    pub episodes: Vec<Episode>,
}
