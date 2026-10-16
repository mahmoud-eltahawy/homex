//! Domain newtypes.
//!
//! A `String` that reaches a domain boundary is parsed into one of these
//! types immediately. From that point on, the type system carries the
//! invariant, and the rest of the codebase never has to wonder whether a
//! value is valid — the same discipline [`Slug`](crate::app::ids::Slug)
//! already established for section identifiers.
//!
//! # What belongs here
//!
//! Type a string if it has a shape that can be violated, or an equality or
//! ordering rule that isn't the string's own. Leave it as `String` if it is
//! genuinely freeform: user-facing text, error messages, filenames chosen by
//! a client.
//!
//! # The four rules
//!
//! Every type here follows the same shape:
//!
//! 1. **Validating constructor.** `new` returns `Result<_, _Error>`.
//! 2. **Trusting `from_stored`.** `pub(crate)` only, for values that
//!    originated from this app's own writes.
//! 3. **`FromStr`** for the domain type's own parse path.
//! 4. **Validating `Deserialize`.** Every value crossing the wire runs
//!    through the constructor.

#![allow(dead_code)]

mod file_ext;
mod job_id;
mod poster_url;
mod rel_path;
mod tag_name;

pub use file_ext::FileExt;
pub use job_id::JobId;
pub use poster_url::PosterUrl;
pub use rel_path::RelPath;
pub use tag_name::TagName;
