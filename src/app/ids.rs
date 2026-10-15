//! Strongly-typed identifiers and small domain newtypes.
//!
//! A `String` that reaches a domain boundary is parsed into one of these
//! types immediately. From that point on, the type system carries the
//! invariant, and the rest of the codebase never has to wonder whether a
//! value is a valid slug.

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display};

// ─── Slug ─────────────────────────────────────────────────────────────────

/// A URL-safe section identifier.
///
/// Invariants:
/// - Non-empty.
/// - Characters limited to `[a-z0-9-]`.
/// - Does not begin or end with `-`.
/// - Contains no `--`.
///
/// [`Slug::new`] enforces these; the `Deserialize` impl routes every
/// inbound value through `new`, so a slug arriving from JSON or a URL is
/// already valid by the time it reaches a server function.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
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
    ///
    /// Every slug written to the database passed through [`Slug::new`], so
    /// the invariant holds. Skips re-validation to keep row decoding cheap.
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

impl From<Slug> for String {
    fn from(s: Slug) -> Self {
        s.0
    }
}
