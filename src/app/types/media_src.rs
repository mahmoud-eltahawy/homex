//! The URL a media file is streamed from.

use std::fmt::{self, Display};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::constants::routes;

const PREFIX: &str = routes::MEDIA_URL_PREFIX;

/// A URL of the form `/media/<uuid>`.
///
/// The stream route is the only thing that produces these; the media player
/// is the only thing that consumes them. Everything between is a `String`
/// that must be formatted, parsed, or compared by hand. With the type, the
/// shape travels with the value.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct MediaSrc(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MediaSrcError;

impl Display for MediaSrcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not a `{PREFIX}` URL")
    }
}

impl std::error::Error for MediaSrcError {}

impl MediaSrc {
    /// Build the URL for one file.
    pub fn for_file(file_id: Uuid) -> Self {
        MediaSrc(format!("{PREFIX}{file_id}"))
    }

    /// The file id this URL points at.
    pub fn file_id(&self) -> Uuid {
        Uuid::parse_str(&self.0[PREFIX.len()..]).expect("MediaSrc invariant: body is a UUID")
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for MediaSrc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for MediaSrc {
    type Err = MediaSrcError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rest = s.strip_prefix(PREFIX).ok_or(MediaSrcError)?;
        Uuid::parse_str(rest).map_err(|_| MediaSrcError)?;
        Ok(MediaSrc(s.to_string()))
    }
}

impl AsRef<str> for MediaSrc {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for MediaSrc {
    type Error = MediaSrcError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.as_str().parse()
    }
}

impl From<MediaSrc> for String {
    fn from(m: MediaSrc) -> String {
        m.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips() {
        let id = Uuid::new_v4();
        let src = MediaSrc::for_file(id);
        assert_eq!(src.file_id(), id);
        assert_eq!(src.as_str(), format!("/media/{id}"));
        let back: MediaSrc = src.as_str().parse().unwrap();
        assert_eq!(back, src);
    }

    #[test]
    fn rejects_other_urls() {
        assert!("/posters/x/y.jpg".parse::<MediaSrc>().is_err());
        assert!("/media/not-a-uuid".parse::<MediaSrc>().is_err());
    }

    #[test]
    fn serde_roundtrip() {
        let src = MediaSrc::for_file(Uuid::new_v4());
        let s = serde_json::to_string(&src).unwrap();
        let back: MediaSrc = serde_json::from_str(&s).unwrap();
        assert_eq!(back, src);
    }
}
