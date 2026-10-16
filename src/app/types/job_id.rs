//! A background job identifier.

use std::fmt::{self, Display};
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A background job identifier.
///
/// Wraps a [`Uuid`]. Used as the key of the job map, as the parameter of
/// polling and cancellation, and as a wire field. `Display` and `FromStr`
/// use the canonical UUID string form.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct JobId(Uuid);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JobIdError;

impl Display for JobIdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("not a valid job id")
    }
}

impl std::error::Error for JobIdError {}

impl JobId {
    /// A fresh, randomly-generated identifier.
    pub fn new_v4() -> Self {
        JobId(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for JobId {
    type Err = JobIdError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::parse_str(s).map(JobId).map_err(|_| JobIdError)
    }
}

impl TryFrom<String> for JobId {
    type Error = JobIdError;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<JobId> for String {
    fn from(j: JobId) -> String {
        j.0.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_string() {
        let id = JobId::new_v4();
        let s = id.to_string();
        let back: JobId = s.parse().unwrap();
        assert_eq!(id, back);
    }

    #[test]
    fn rejects_garbage() {
        assert!("not-a-uuid".parse::<JobId>().is_err());
    }

    #[test]
    fn serde_roundtrip() {
        let id = JobId::new_v4();
        let s = serde_json::to_string(&id).unwrap();
        let back: JobId = serde_json::from_str(&s).unwrap();
        assert_eq!(back, id);
    }
}
