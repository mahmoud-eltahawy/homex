//! A user-facing title.

use std::fmt::{self, Display};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A section, collection, or item title.
///
/// Invariants:
/// - Non-empty after trimming.
/// - Trimmed on construction.
///
/// "Empty title" was rejected in five different places — `create_section`,
/// `update_section`, `patch_section_title`, `patch_collection_field`, and
/// the group-classify flow — each with its own `if title.trim().is_empty()`
/// check. The type carries the rule once.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Title(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TitleError;

impl Display for TitleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("title is empty")
    }
}

impl std::error::Error for TitleError {}

impl Title {
    /// Validate and construct. The input is trimmed first.
    pub fn new(raw: impl Into<String>) -> Result<Self, TitleError> {
        let s = raw.into().trim().to_string();
        if s.is_empty() {
            return Err(TitleError);
        }
        Ok(Title(s))
    }

    /// Construct from a value that originated from our own database.
    ///
    /// Every title written to the database passed through [`Title::new`].
    #[allow(dead_code)] // Called only by ssr-gated server code.
    pub(crate) fn from_stored(raw: String) -> Self {
        Title(raw)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl Display for Title {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for Title {
    type Err = TitleError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Title::new(s)
    }
}

impl AsRef<str> for Title {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Title {
    type Error = TitleError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Title::new(s)
    }
}

impl From<Title> for String {
    fn from(t: Title) -> String {
        t.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_rejects_empty() {
        assert_eq!(Title::new("  Hello  ").unwrap().as_str(), "Hello");
        assert!(Title::new("").is_err());
        assert!(Title::new("   \t\n").is_err());
    }

    #[test]
    fn serde_rejects_empty() {
        let err = serde_json::from_str::<Title>(r#""   ""#).unwrap_err();
        assert!(err.to_string().contains("title is empty"));
    }

    #[test]
    fn serde_roundtrip_preserves_trimmed() {
        let t = Title::new("  Dune  ").unwrap();
        let s = serde_json::to_string(&t).unwrap();
        assert_eq!(s, r#""Dune""#);
        let back: Title = serde_json::from_str(&s).unwrap();
        assert_eq!(back, t);
    }

    #[test]
    fn from_str_is_symmetric_with_display() {
        let t: Title = "  Apple  ".parse().unwrap();
        assert_eq!(t.to_string(), "Apple");
    }
}
