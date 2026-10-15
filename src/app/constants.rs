//! Shared string constants: route paths, storage layout, environment
//! variables, HTTP protocol prefixes, static-asset extensions, and
//! reusable server messages.

// ─── App metadata ─────────────────────────────────────────────────────────

pub const APP_TITLE: &str = "HomeX";
pub const HTML_LANG: &str = "en";
pub const HTML_DIR: &str = "ltr";
pub const STYLESHEET_URL: &str = "/pkg/homex.css";

// ─── Routes ───────────────────────────────────────────────────────────────

pub mod routes {
    use uuid::Uuid;

    pub const HOME: &str = "/";
    pub const LOGIN: &str = "/login";
    pub const LOGIN_API: &str = "/api/login";

    pub const SECTION_PREFIX: &str = "/s/";

    pub const MEDIA_URL_PREFIX: &str = "/media/";
    pub const MEDIA_STREAM_PATTERN: &str = "/media/{id}";

    pub const POSTERS_URL_PREFIX: &str = "/posters/";
    pub const POSTERS_SERVE_PREFIX: &str = "/posters";

    pub const PKG_URL_PREFIX: &str = "/pkg/";

    pub fn section(slug: &str) -> String {
        format!("{SECTION_PREFIX}{slug}")
    }
    pub fn collection(slug: &str, id: Uuid) -> String {
        format!("{SECTION_PREFIX}{slug}/{id}")
    }
    pub fn item(slug: &str, collection_id: Uuid, item_id: Uuid) -> String {
        format!("{SECTION_PREFIX}{slug}/{collection_id}/item/{item_id}")
    }
    pub fn media(file_id: Uuid) -> String {
        format!("{MEDIA_URL_PREFIX}{file_id}")
    }
    pub fn poster(subdir: &str, filename: &str) -> String {
        format!("{POSTERS_URL_PREFIX}{subdir}/{filename}")
    }

    pub const UNCLASSIFIED: &str = "/unclassified";
}

// ─── Storage layout ───────────────────────────────────────────────────────

pub mod storage {
    pub const POSTERS_DIR: &str = "posters";
    pub const DB_FILENAME: &str = "homex.db";
    pub const UPLOAD_TEMP_DIR: &str = "homex-uploads";
}

// ─── Auth ─────────────────────────────────────────────────────────────────

pub mod auth {
    pub const COOKIE_NAME: &str = "homex_token";
    pub const COOKIE_PREFIX: &str = "homex_token=";
    pub const COOKIE_ATTRS: &str = "; Path=/; HttpOnly; SameSite=Lax; Max-Age=2592000";
    pub const BEARER_PREFIX: &str = "Bearer ";
    pub const ENV_TOKEN: &str = "HOMEX_TOKEN";
}

// ─── Config ───────────────────────────────────────────────────────────────

pub mod config {
    pub const ENV_CONFIG: &str = "HOMEX_CONFIG";
    pub const ENV_RESET_SCHEMA: &str = "HOMEX_RESET_SCHEMA";
    pub const DEFAULT_PATH: &str = "homex.toml";
    pub const ENV_UNCLASSIFIED_DIR: &str = "HOMEX_UNCLASSIFIED_DIR";
}

// ─── Protocol prefixes ────────────────────────────────────────────────────

pub mod protocols {
    pub const HTTP: &str = "http://";
    pub const HTTPS: &str = "https://";
}

// ─── Static-asset extensions ──────────────────────────────────────────────

pub const STATIC_ASSET_EXTS: &[&str] = &[
    "css", "js", "wasm", "png", "jpg", "jpeg", "webp", "svg", "ico", "woff", "woff2",
];

// ─── Server messages ──────────────────────────────────────────────────────
//
// Every string here is the display text for exactly one `ErrorCode`. The
// mapping lives in `crate::app::error::AppError::user_message`.

pub mod messages {
    pub const INTERNAL_ERROR: &str = "Internal error";

    // Resources
    pub const COLLECTION_NOT_FOUND: &str = "Collection not found";
    pub const SECTION_NOT_FOUND: &str = "Section not found";
    pub const FILE_NOT_FOUND: &str = "File not found";
    pub const JOB_NOT_FOUND: &str = "Job not found";

    // Validation
    pub const TITLE_REQUIRED: &str = "Title is required";
    pub const NAME_REQUIRED: &str = "Name is required";
    pub const INVALID_TOKEN: &str = "Invalid token";
    pub const INVALID_PATH: &str = "Invalid path";

    // Upload
    pub const NO_FILES_RECEIVED: &str = "No files were received";
    pub const NO_IMAGE_RECEIVED: &str = "No image was received";
    pub const SECTION_SLUG_MISSING: &str = "section_slug is missing";
    pub const UPLOAD_TARGET_MISSING: &str = "Either collection_id or collection_title is required";
    pub const MEDIA_KIND_MISMATCH: &str =
        "Cannot move a collection across media kinds (video ↔ audio)";
    pub const UNSUPPORTED_FORMAT: &str = "Unsupported media format";

    // Sections
    pub const SECTION_NOT_EMPTY: &str =
        "Section is not empty — move or delete its collections first";
    pub const UNIQUE_SECTION_ID_FAILED: &str = "Could not generate a unique section identifier";

    // Classify
    pub const CLASSIFICATION_IN_PROGRESS: &str =
        "A classification is already running for one of these paths";
    pub const NO_FOLDERS_SELECTED: &str = "No folders selected";
    pub const NO_FILES_FOUND: &str = "No media files found";
    pub const NOT_A_FOLDER: &str = "Not a folder";
    pub const NOT_A_FILE: &str = "Not a file";

    // Upload progress
    pub const UPLOAD_STARTED_BG: &str = "Upload and conversion started in the background";
}
