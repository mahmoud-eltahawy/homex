//! A tag name.

use std::cmp::Ordering;
use std::fmt::{self, Display};
use std::hash::{Hash, Hasher};
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A tag name.
///
/// Invariants:
/// - Non-empty after trimming.
///
/// `PartialEq`, `Eq`, `Hash`, `PartialOrd`, and `Ord` are all **case-insensitive**.
/// The current codebase enforces that rule in three separate places
/// (`canonicalize_tags`, `ci_eq`, and the `COLLATE NOCASE` clause in
/// `find_tag_id_by_name`). With the type, the rule lives in exactly one place,
/// and every `Vec::contains`, `HashSet::insert`, and `sort` is case-insensitive
/// by construction.
#[derive(Clone, Debug, Serialize)]
#[serde(transparent)]
pub struct TagName(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TagNameError;

impl Display for TagNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("tag name is empty")
    }
}

impl std::error::Error for TagNameError {}

impl TagName {
    pub fn new(raw: impl Into<String>) -> Result<Self, TagNameError> {
        let s = raw.into().trim().to_string();
        if s.is_empty() {
            return Err(TagNameError);
        }
        Ok(TagName(s))
    }

    /// Construct from a value that originated from our own database.
    pub(crate) fn from_stored(raw: String) -> Self {
        TagName(raw)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The case-folded form used for every comparison.
    fn fold(&self) -> String {
        self.0.to_lowercase()
    }
}

impl PartialEq for TagName {
    fn eq(&self, other: &Self) -> bool {
        self.fold() == other.fold()
    }
}

impl Eq for TagName {}

impl Hash for TagName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.fold().hash(state);
    }
}

impl PartialOrd for TagName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TagName {
    fn cmp(&self, other: &Self) -> Ordering {
        self.fold().cmp(&other.fold())
    }
}

impl Display for TagName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for TagName {
    type Err = TagNameError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TagName::new(s)
    }
}

impl AsRef<str> for TagName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for TagName {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trims_and_rejects_empty() {
        assert_eq!(TagName::new("  action  ").unwrap().as_str(), "action");
        assert!(TagName::new("").is_err());
        assert!(TagName::new("   ").is_err());
    }

    #[test]
    fn equality_is_case_insensitive() {
        assert_eq!(
            TagName::new("Action").unwrap(),
            TagName::new("ACTION").unwrap()
        );
    }

    #[test]
    fn hash_is_case_insensitive() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(TagName::new("Action").unwrap());
        assert!(set.contains(&TagName::new("ACTION").unwrap()));
    }

    #[test]
    fn sorting_is_case_insensitive() {
        let mut v = vec![
            TagName::new("zebra").unwrap(),
            TagName::new("APPLE").unwrap(),
            TagName::new("Mango").unwrap(),
        ];
        v.sort();
        assert_eq!(v[0].as_str(), "APPLE");
        assert_eq!(v[1].as_str(), "Mango");
        assert_eq!(v[2].as_str(), "zebra");
    }
}
