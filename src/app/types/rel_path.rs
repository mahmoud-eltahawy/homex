//! A workspace-relative media path.

use std::fmt::{self, Display};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A workspace-relative path.
///
/// Invariants:
/// - No NUL byte.
/// - No component is empty, `.`, or `..`.
/// - No leading or trailing `/` is stored (they are stripped on construction).
///
/// The invariant is what makes it safe to join onto a trusted root: no
/// component of the path can escape that root. [`RelPath::join_under`] is
/// the *only* way the codebase reaches a file on disk from a stored path.
///
/// An empty `RelPath` is valid and refers to the root itself. That covers
/// the "unclassified folder at the top level" case, which is a legitimate
/// empty path, not a malformed one.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct RelPath(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RelPathError {
    /// A path component is empty, `.`, or `..`.
    BadComponent(String),
    /// The path contains a NUL byte.
    NulByte,
}

impl Display for RelPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadComponent(s) => write!(f, "invalid path component: {s:?}"),
            Self::NulByte => f.write_str("path contains a NUL byte"),
        }
    }
}

impl std::error::Error for RelPathError {}

impl RelPath {
    /// Validate and construct.
    ///
    /// Leading and trailing `/` are stripped. The remaining path is split on
    /// `/` and every component is checked to be non-empty, not `.`, and not
    /// `..`. The empty result is valid and refers to the root.
    pub fn new(raw: impl Into<String>) -> Result<Self, RelPathError> {
        let s = raw.into();
        if s.contains('\0') {
            return Err(RelPathError::NulByte);
        }
        let trimmed = s.trim_matches('/');
        if trimmed.is_empty() {
            return Ok(RelPath(String::new()));
        }
        for part in trimmed.split('/') {
            if part.is_empty() || part == "." || part == ".." {
                return Err(RelPathError::BadComponent(part.to_string()));
            }
        }
        Ok(RelPath(trimmed.to_string()))
    }

    /// Construct from a value that originated from our own database.
    ///
    /// Every path written to the database passed through [`RelPath::new`],
    /// so the invariant holds. Skips re-validation to keep row decoding
    /// cheap.
    pub(crate) fn from_stored(raw: String) -> Self {
        RelPath(raw)
    }

    /// Join this path under a trusted root.
    ///
    /// This is the only way the codebase ever reaches a file on disk from a
    /// stored path. The type's invariant guarantees the result is inside
    /// `root`, so the caller does not need a `starts_with` check.
    pub fn join_under(&self, root: &Path) -> PathBuf {
        if self.0.is_empty() {
            root.to_path_buf()
        } else {
            root.join(&self.0)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl Display for RelPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for RelPath {
    type Err = RelPathError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        RelPath::new(s)
    }
}

impl AsRef<str> for RelPath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for RelPath {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        RelPath::new(s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_valid_and_refers_to_root() {
        let r = RelPath::new("").unwrap();
        assert_eq!(r.as_str(), "");
        assert!(r.is_empty());
    }

    #[test]
    fn leading_and_trailing_slashes_are_stripped() {
        assert_eq!(RelPath::new("/a/b/").unwrap().as_str(), "a/b");
        assert_eq!(RelPath::new("///a/b///").unwrap().as_str(), "a/b");
    }

    #[test]
    fn parent_segments_are_rejected() {
        assert!(RelPath::new("../etc/passwd").is_err());
        assert!(RelPath::new("a/../../b").is_err());
        assert!(RelPath::new("a/./b").is_err());
        assert!(RelPath::new("a//b").is_err());
    }

    #[test]
    fn nul_is_rejected() {
        assert!(RelPath::new("a\0b").is_err());
    }

    #[test]
    fn join_under_never_escapes_root() {
        let root = Path::new("/srv/homex/media");
        assert_eq!(
            RelPath::new("movies/a.mp4").unwrap().join_under(root),
            root.join("movies/a.mp4"),
        );
        assert_eq!(RelPath::new("").unwrap().join_under(root), root);
    }
}
