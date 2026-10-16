//! Strongly-typed identifiers and small domain newtypes.

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

/// A URL-safe section identifier.
///
/// Invariants:
/// - Non-empty.
/// - Characters limited to `[a-z0-9-]`.
/// - Does not begin or end with `-`.
/// - Contains no `--`.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Slug(String);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SlugError {
    Empty,
    InvalidChar(char),
    BoundaryHyphen,
    ConsecutiveHyphen,
}

impl Display for SlugError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SlugError::Empty => f.write_str("slug is empty"),
            SlugError::InvalidChar(c) => {
                write!(f, "slug contains an invalid character: {c:?}")
            }
            SlugError::BoundaryHyphen => f.write_str("slug must not start or end with '-'"),
            SlugError::ConsecutiveHyphen => f.write_str("slug must not contain consecutive '-'"),
        }
    }
}

impl std::error::Error for SlugError {}

impl Slug {
    /// Validate and construct.
    pub fn new(raw: impl Into<String>) -> Result<Self, SlugError> {
        let s = raw.into();
        if s.is_empty() {
            return Err(SlugError::Empty);
        }
        if s.starts_with('-') || s.ends_with('-') {
            return Err(SlugError::BoundaryHyphen);
        }
        if s.contains("--") {
            return Err(SlugError::ConsecutiveHyphen);
        }
        for c in s.chars() {
            if !(c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
                return Err(SlugError::InvalidChar(c));
            }
        }
        Ok(Slug(s))
    }

    /// Construct from a value that originated from our own database.
    #[allow(dead_code)] // Called only by ssr-gated server code.
    pub(crate) fn from_stored(raw: String) -> Self {
        Slug(raw)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for Slug {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::str::FromStr for Slug {
    type Err = SlugError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Slug::new(s)
    }
}

impl AsRef<str> for Slug {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Slug {
    type Error = SlugError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Slug::new(s)
    }
}

impl From<Slug> for String {
    fn from(s: Slug) -> String {
        s.0
    }
}
