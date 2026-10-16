//! A file extension.

use std::fmt::{self, Display};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Maximum extension length accepted.
const MAX_LEN: usize = 8;

/// A file extension.
///
/// Invariants:
/// - Length `1..=8`.
/// - Every character is ASCII alphanumeric.
/// - Stored lowercase.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct FileExt(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileExtError {
    Empty,
    TooLong,
    NonAlphanumeric(char),
}

impl Display for FileExtError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("extension is empty"),
            Self::TooLong => write!(f, "extension is longer than {MAX_LEN} characters"),
            Self::NonAlphanumeric(c) => {
                write!(f, "extension contains a non-alphanumeric character: {c:?}")
            }
        }
    }
}

impl std::error::Error for FileExtError {}

impl FileExt {
    /// Validate and construct. The input is lowercased first.
    pub fn new(raw: impl Into<String>) -> Result<Self, FileExtError> {
        let s = raw.into().to_ascii_lowercase();
        if s.is_empty() {
            return Err(FileExtError::Empty);
        }
        if s.len() > MAX_LEN {
            return Err(FileExtError::TooLong);
        }
        for c in s.chars() {
            if !c.is_ascii_alphanumeric() {
                return Err(FileExtError::NonAlphanumeric(c));
            }
        }
        Ok(FileExt(s))
    }

    /// Extract the extension from a filename.
    pub fn from_filename(name: &str) -> Option<Self> {
        let (_, ext) = name.rsplit_once('.')?;
        FileExt::new(ext).ok()
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for FileExt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for FileExt {
    type Err = FileExtError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FileExt::new(s)
    }
}

impl AsRef<str> for FileExt {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for FileExt {
    type Error = FileExtError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        FileExt::new(s)
    }
}

impl From<FileExt> for String {
    fn from(e: FileExt) -> String {
        e.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowercases_on_construction() {
        assert_eq!(FileExt::new("MP4").unwrap().as_str(), "mp4");
        assert_eq!(FileExt::new("JpEg").unwrap().as_str(), "jpeg");
    }

    #[test]
    fn rejects_empty_and_too_long() {
        assert!(FileExt::new("").is_err());
        assert!(FileExt::new("a".repeat(MAX_LEN + 1)).is_err());
    }

    #[test]
    fn rejects_non_alphanumeric() {
        assert!(FileExt::new("m-p4").is_err());
        assert!(FileExt::new("m.p4").is_err());
    }

    #[test]
    fn extracts_from_filename() {
        assert_eq!(FileExt::from_filename("movie.MP4").unwrap().as_str(), "mp4");
        assert!(FileExt::from_filename("no-extension").is_none());
    }
}
