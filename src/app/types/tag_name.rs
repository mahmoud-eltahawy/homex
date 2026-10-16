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
/// Comparison uses an ASCII fast path with a `to_lowercase()` fallback for
/// non-ASCII input. The fallback is authoritative — the fast paths are only
/// used when both sides are ASCII, where they are provably equivalent.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
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
    #[allow(dead_code)] // Called only by ssr-gated server code.
    pub(crate) fn from_stored(raw: String) -> Self {
        TagName(raw)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The case-folded form used as the authoritative comparison.
    fn fold(&self) -> String {
        self.0.to_lowercase()
    }
}

impl PartialEq for TagName {
    fn eq(&self, other: &Self) -> bool {
        if self.0.is_ascii() && other.0.is_ascii() {
            return self.0.eq_ignore_ascii_case(&other.0);
        }
        self.fold() == other.fold()
    }
}

impl Eq for TagName {}

impl Hash for TagName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Fast path: ASCII-only, fits in a 128-byte stack buffer.
        if self.0.is_ascii() && self.0.len() <= 128 {
            let mut buf = [0u8; 128];
            for (i, b) in self.0.bytes().enumerate() {
                buf[i] = b.to_ascii_lowercase();
            }
            // Safe: all bytes are ASCII (from `is_ascii`).
            let folded =
                std::str::from_utf8(&buf[..self.0.len()]).expect("ASCII bytes are valid UTF-8");
            folded.hash(state);
            return;
        }
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
        if self.0.is_ascii() && other.0.is_ascii() {
            return self
                .0
                .bytes()
                .map(|b| b.to_ascii_lowercase())
                .cmp(other.0.bytes().map(|b| b.to_ascii_lowercase()));
        }
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

impl TryFrom<String> for TagName {
    type Error = TagNameError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        TagName::new(s)
    }
}

impl From<TagName> for String {
    fn from(t: TagName) -> String {
        t.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::hash::DefaultHasher;

    fn hash_of(t: &TagName) -> u64 {
        let mut h = DefaultHasher::new();
        t.hash(&mut h);
        h.finish()
    }

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

    #[test]
    fn ascii_fast_path_matches_folded_behavior() {
        let a = TagName::new("Action").unwrap();
        let b = TagName::new("ACTION").unwrap();
        assert_eq!(a, b);
        assert_eq!(hash_of(&a), hash_of(&b));

        // Ordered under folding but not under byte order.
        let lower = TagName::new("apple").unwrap();
        let upper = TagName::new("Banana").unwrap();
        assert!(lower < upper, "folded: apple < banana");
        assert!("apple" > "Banana", "byte order would say otherwise");

        let mut set = HashSet::new();
        set.insert(TagName::new("Action").unwrap());
        assert!(set.contains(&TagName::new("ACTION").unwrap()));
        assert!(!set.contains(&TagName::new("Drama").unwrap()));
    }

    #[test]
    fn non_ascii_comparison_is_case_insensitive() {
        let a = TagName::new("Äpfel").unwrap();
        let b = TagName::new("äPFEL").unwrap();
        assert_eq!(a, b);
        assert_eq!(hash_of(&a), hash_of(&b));
        assert_eq!(a.cmp(&b), Ordering::Equal);

        let mut set = HashSet::new();
        set.insert(a);
        assert!(set.contains(&b));
    }
}
