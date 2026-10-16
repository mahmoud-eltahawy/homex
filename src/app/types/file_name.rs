//! A filename (a single path component naming a file).

use std::fmt::{self, Display};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use super::file_ext::FileExt;

/// A single path component naming a file.
///
/// Invariants:
/// - Non-empty.
/// - Contains no `/`, `\`, or NUL.
///
/// A filename is not a path. `sanitize_filename`, `stem_of`, `safe_stem`,
/// and `extension_of` existed as separate helpers because the shape was
/// never captured. With the type, `stem()` and `extension()` are methods,
/// not free functions, and `FileName::new` refuses the characters the old
/// `sanitize_filename` was stripping.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct FileName(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FileNameError {
    Empty,
    Separator,
    NulByte,
}

impl Display for FileNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("filename is empty"),
            Self::Separator => f.write_str("filename contains a path separator"),
            Self::NulByte => f.write_str("filename contains a NUL byte"),
        }
    }
}

impl std::error::Error for FileNameError {}

impl FileName {
    pub fn new(raw: impl Into<String>) -> Result<Self, FileNameError> {
        let s = raw.into();
        if s.is_empty() {
            return Err(FileNameError::Empty);
        }
        if s.contains('\0') {
            return Err(FileNameError::NulByte);
        }
        if s.contains('/') || s.contains('\\') {
            return Err(FileNameError::Separator);
        }
        Ok(FileName(s))
    }

    /// Construct from a value that originated from our own filesystem reads.
    ///
    /// Filesystem entry names cannot contain `/` or NUL on any platform we
    /// support, so the invariant holds by construction.
    #[allow(dead_code)] // Called only by ssr-gated server code.
    pub(crate) fn from_stored(raw: String) -> Self {
        FileName(raw)
    }

    /// The extension of this filename, if any.
    pub fn extension(&self) -> Option<FileExt> {
        FileExt::from_filename(&self.0)
    }

    /// The stem of this filename (everything before the last `.`).
    pub fn stem(&self) -> &str {
        self.0.rsplit_once('.').map(|(s, _)| s).unwrap_or(&self.0)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl Display for FileName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for FileName {
    type Err = FileNameError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        FileName::new(s)
    }
}

impl AsRef<str> for FileName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for FileName {
    type Error = FileNameError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        FileName::new(s)
    }
}

impl From<FileName> for String {
    fn from(f: FileName) -> String {
        f.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_separators_and_empty() {
        assert!(FileName::new("a/b").is_err());
        assert!(FileName::new("a\\b").is_err());
        assert!(FileName::new("").is_err());
        assert!(FileName::new("a\0b").is_err());
    }

    #[test]
    fn stem_and_extension() {
        let f = FileName::new("movie.MP4").unwrap();
        assert_eq!(f.stem(), "movie");
        assert_eq!(f.extension().unwrap().as_str(), "mp4");

        let f = FileName::new("README").unwrap();
        assert_eq!(f.stem(), "README");
        assert!(f.extension().is_none());
    }

    #[test]
    fn serde_rejects_separator() {
        assert!(serde_json::from_str::<FileName>(r#""a/b""#).is_err());
    }

    #[test]
    fn serde_roundtrip() {
        let f = FileName::new("movie.mp4").unwrap();
        let s = serde_json::to_string(&f).unwrap();
        assert_eq!(s, r#""movie.mp4""#);
        let back: FileName = serde_json::from_str(&s).unwrap();
        assert_eq!(back, f);
    }
}
