pub const SECTION_SLUG: &str = "section_slug";
pub const COLLECTION_ID: &str = "collection_id";
pub const SEASON_NUMBER: &str = "season_number";

pub const FILE_PREFIX: &str = "file_";
pub const FILE_TITLE_PREFIX: &str = "file_title_";

pub fn file(idx: usize) -> String {
    format!("{FILE_PREFIX}{idx}")
}

pub fn file_title(idx: usize) -> String {
    format!("{FILE_TITLE_PREFIX}{idx}")
}
