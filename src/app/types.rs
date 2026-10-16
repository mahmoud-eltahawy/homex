//! Domain newtypes.
//!
//! A `String` that reaches a domain boundary is parsed into one of these
//! types immediately. From that point on, the type system carries the
//! invariant, and the rest of the codebase never has to wonder whether a
//! value is valid.
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
//!    through the constructor. Enforced by `#[serde(try_from = "String")]`
//!    plus a `TryFrom<String>` impl, not by a hand-written visitor.

mod file_ext;
mod file_name;
mod job_id;
mod media_src;
mod poster_url;
mod rel_path;
mod tag_name;
mod title;

pub use file_ext::FileExt;
pub use file_name::FileName;
pub use job_id::JobId;
pub use media_src::MediaSrc;
pub use poster_url::PosterUrl;
pub use rel_path::RelPath;
pub use tag_name::TagName;
pub use title::Title;
