//! A poster URL.

use std::fmt::{self, Display};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::app::constants::{routes, storage};

const PREFIX: &str = routes::POSTERS_URL_PREFIX;

/// A URL of the form `/posters/<subdir>/<filename>`.
///
/// Invariants:
/// - Starts with `/posters/`.
/// - `<subdir>` is non-empty, contains no `/`.
/// - `<filename>` is non-empty, contains no `/`.
///
/// The prefix is what every consumer of a raw poster string had to strip and
/// then re-check, in three places, each with its own error handling. With
/// the type, the prefix is a fact of the value: `subdir()` and `filename()`
/// return the parsed parts, and `to_filesystem_path()` gives the on-disk
/// location.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct PosterUrl(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PosterUrlError {
    /// Missing the `/posters/` prefix.
    MissingPrefix,
    /// There is no `/` after the prefix.
    MissingSeparator,
    /// `subdir` or `filename` is empty.
    EmptyComponent,
}

impl Display for PosterUrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingPrefix => write!(f, "missing `{PREFIX}` prefix"),
            Self::MissingSeparator => {
                write!(f, "missing `/` separating subdir from filename")
            }
            Self::EmptyComponent => f.write_str("empty subdir or filename"),
        }
    }
}

impl std::error::Error for PosterUrlError {}

impl PosterUrl {
    /// Build one from its parts.
    ///
    /// `subdir` and `filename` must be non-empty and free of `/`. `subdir` is
    /// typically a section slug, which is already validated elsewhere, but
    /// this constructor does not assume that.
    pub fn for_file(subdir: &str, filename: &str) -> Result<Self, PosterUrlError> {
        if subdir.is_empty() || filename.is_empty() {
            return Err(PosterUrlError::EmptyComponent);
        }
        if subdir.contains('/') || filename.contains('/') {
            return Err(PosterUrlError::EmptyComponent);
        }
        Ok(PosterUrl(format!("{PREFIX}{subdir}/{filename}")))
    }

    /// Construct from a value that originated from our own database.
    ///
    /// Every URL written to the database passed through [`PosterUrl::for_file`]
    /// or was produced by a path that did. Skips re-validation.
    pub(crate) fn from_stored(raw: String) -> Self {
        PosterUrl(raw)
    }

    /// The path component after `/posters/`, before the second `/`.
    pub fn subdir(&self) -> &str {
        let rest = &self.0[PREFIX.len()..];
        rest.split_once('/').map(|(s, _)| s).unwrap_or(rest)
    }

    /// The final path component.
    pub fn filename(&self) -> &str {
        let rest = &self.0[PREFIX.len()..];
        rest.rsplit_once('/').map(|(_, f)| f).unwrap_or(rest)
    }

    /// Where this poster lives on disk under `data_dir`.
    pub fn to_filesystem_path(&self, data_dir: &Path) -> PathBuf {
        data_dir
            .join(storage::POSTERS_DIR)
            .join(self.subdir())
            .join(self.filename())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for PosterUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for PosterUrl {
    type Err = PosterUrlError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let rest = s
            .strip_prefix(PREFIX)
            .ok_or(PosterUrlError::MissingPrefix)?;
        let (subdir, filename) = rest
            .split_once('/')
            .ok_or(PosterUrlError::MissingSeparator)?;
        PosterUrl::for_file(subdir, filename)
    }
}

impl AsRef<str> for PosterUrl {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for PosterUrl {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_from_parts() {
        let u = PosterUrl::for_file("movies", "abc.jpg").unwrap();
        assert_eq!(u.as_str(), "/posters/movies/abc.jpg");
        assert_eq!(u.subdir(), "movies");
        assert_eq!(u.filename(), "abc.jpg");
    }

    #[test]
    fn rejects_empty_parts() {
        assert!(PosterUrl::for_file("", "a.jpg").is_err());
        assert!(PosterUrl::for_file("movies", "").is_err());
        assert!(PosterUrl::for_file("a/b", "c.jpg").is_err());
    }

    #[test]
    fn parses_a_full_url() {
        let u: PosterUrl = "/posters/movies/abc.jpg".parse().unwrap();
        assert_eq!(u.subdir(), "movies");
        assert_eq!(u.filename(), "abc.jpg");
    }

    #[test]
    fn rejects_missing_prefix() {
        assert!("/media/abc.jpg".parse::<PosterUrl>().is_err());
    }
}
