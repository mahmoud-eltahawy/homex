use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MediaId(pub i64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileSize(pub u64);

impl FileSize {
    pub fn human_readable(&self) -> String {
        let bytes = self.0 as f64;
        if bytes >= 1_000_000_000.0 {
            format!("{:.1} جيجابايت", bytes / 1_000_000_000.0)
        } else if bytes >= 1_000_000.0 {
            format!("{:.1} ميجابايت", bytes / 1_000_000.0)
        } else if bytes >= 1_000.0 {
            format!("{:.1} كيلوبايت", bytes / 1_000.0)
        } else {
            format!("{} بايت", bytes)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurationSeconds(pub u64);

impl DurationSeconds {
    pub fn human_readable(&self) -> String {
        let secs = self.0;
        let hours = secs / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;
        if hours > 0 {
            format!("{} ساعة و{} دقيقة", hours, minutes)
        } else if minutes > 0 {
            format!("{} دقيقة", minutes)
        } else {
            format!("{} ثانية", seconds)
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MediaFile {
    pub path: String,
    pub size: FileSize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Movie {
    pub id: MediaId,
    pub title: String,
    pub poster: Option<String>,
    pub description: Option<String>,
    pub file: MediaFile,
    pub duration: DurationSeconds,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AudioGroup {
    pub id: MediaId,
    pub title: String,
    pub poster: Option<String>,
    pub description: Option<String>,
    pub audios_count: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum MediaType {
    Movie,
    Series,
    AudioGroup,
}

impl std::fmt::Display for MediaType {
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
    pub duration: DurationSeconds,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Series {
    pub id: MediaId,
    pub title: String,
    pub poster: Option<String>,
    pub description: Option<String>,
    pub season_count: u32,
    pub season_summaries: Vec<SeasonSummary>,
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
