//! Route parameter readers and [`RouteError`].
//!
//! Every reader returns `Result`. A parse failure is not swallowed — it
//! propagates as a [`RouteError`] and short-circuits the page to its
//! `render_error`. There is no default-on-failure value anywhere in this
//! module.

use leptos::reactive::traits::With;
use leptos_router::hooks::use_params_map;
use std::fmt::{self, Display};
use std::str::FromStr;
use uuid::Uuid;

/// A URL path parameter could not be read or parsed.
///
/// This is the `E` in `Page<T, E>` for any page that reads path parameters,
/// and the only failure mode such a page can hit before any resource is
/// constructed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RouteError {
    /// The parameter was not present in the URL at all.
    Missing(&'static str),
    /// The parameter was present but is not a valid UUID.
    InvalidUuid { param: &'static str, value: String },
    /// The parameter was present but `FromStr::from_str` rejected it.
    InvalidValue {
        param: &'static str,
        value: String,
        reason: String,
    },
}

impl Display for RouteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing(param) => {
                write!(f, "missing route parameter `{param}`")
            }
            Self::InvalidUuid { param, value } => {
                write!(f, "route parameter `{param}` is not a UUID: {value:?}")
            }
            Self::InvalidValue {
                param,
                value,
                reason,
            } => write!(
                f,
                "route parameter `{param}` is invalid ({value:?}): {reason}",
            ),
        }
    }
}

impl std::error::Error for RouteError {}

// ─── Readers ──────────────────────────────────────────────────────────────────
//
// `read_raw` is the only thing in this crate that touches `use_params_map`.
// Everything else composes on top of it.

fn read_raw(name: &'static str) -> Option<String> {
    use_params_map().try_with(|p| p.get(name)).flatten()
}

/// Read the named parameter as a raw `String`.
///
/// Fails only if the parameter is absent. Use this when the value is
/// genuinely freeform — a search term, a path fragment with no parse step.
pub fn use_string_param(name: &'static str) -> Result<String, RouteError> {
    read_raw(name).ok_or(RouteError::Missing(name))
}

/// Read the named parameter and parse it via [`FromStr`].
///
/// The domain-agnostic core of the module. A domain type with a validating
/// `FromStr` impl — a slug, an enum, a newtype — plugs straight in without
/// the framework needing to know about it.
pub fn use_parsed_param<T>(name: &'static str) -> Result<T, RouteError>
where
    T: FromStr,
    T::Err: Display,
{
    let raw = read_raw(name).ok_or(RouteError::Missing(name))?;
    raw.parse::<T>().map_err(|e| RouteError::InvalidValue {
        param: name,
        value: raw,
        reason: e.to_string(),
    })
}

/// Read the named parameter and parse it as a [`Uuid`].
pub fn use_uuid_param(name: &'static str) -> Result<Uuid, RouteError> {
    let raw = read_raw(name).ok_or(RouteError::Missing(name))?;
    Uuid::parse_str(&raw).map_err(|_| RouteError::InvalidUuid {
        param: name,
        value: raw,
    })
}

/// Read the named parameter and parse it as a [`Uuid`], if present.
///
/// Returns `Ok(None)` when the parameter is absent — the URL simply didn't
/// match that segment — and `Err` when it is present but malformed. The
/// difference matters: "this URL has no such segment" is not the same as
/// "this URL has such a segment and it's wrong".
pub fn use_optional_uuid_param(name: &'static str) -> Result<Option<Uuid>, RouteError> {
    match read_raw(name) {
        None => Ok(None),
        Some(raw) => Uuid::parse_str(&raw)
            .map(Some)
            .map_err(|_| RouteError::InvalidUuid {
                param: name,
                value: raw,
            }),
    }
}
