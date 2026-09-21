pub struct UploadPayload {
    pub section_slug: String,
    pub collection_id: Option<i64>, // None → create new collection
    pub title: String,
    pub description: String,
    pub season_number: Option<i64>,
    pub files: Vec<UploadFile>,
    pub poster: Option<(String, Vec<u8>)>,
}

pub struct UploadFile {
    pub filename: String,
    pub title: String,
    pub bytes: Vec<u8>,
}
pub struct StagedFile {
    pub rel: String,
    pub duration: i64,
    pub size: u64,
    pub title: String,
}
