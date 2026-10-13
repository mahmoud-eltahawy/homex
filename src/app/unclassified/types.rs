//! Wire types shared between client and server for the unclassified feature.
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::app::model::{CollectionSummary, MediaKind};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnclassifiedEntry {
    pub name: String,
    pub rel_path: String,
    pub is_dir: bool,
    pub size: u64,
    pub ext: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionWithCollections {
    pub id: Uuid,
    pub slug: String,
    pub title: String,
    pub nested: bool,
    pub collections: Vec<CollectionSummary>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClassifyInput {
    pub rel_path: String,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderInspection {
    pub rel_path: String,
    pub name: String,
    pub has_subdirs: bool,
    pub subdir_names: Vec<String>,
    pub file_count: usize,
    pub total_size: u64,
    pub dominant_kind: Option<MediaKind>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FoldersInspection {
    pub folders: Vec<FolderInspection>,
    pub total_files: usize,
    pub total_size: u64,
    pub any_has_subdirs: bool,
    pub common_kind: Option<MediaKind>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FolderDestination {
    Existing { section_slug: String },
    New { section_title: String },
}

/// One file's job. `rel_path` / `title` / `season_number` / `collection_id`
/// let the client reconstruct a retry request without reselection.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FileJobHandle {
    pub name: String,
    pub job_id: String,
    pub section_slug: String,
    pub collection_id: Option<Uuid>,
    pub rel_path: String,
    pub title: String,
    pub season_number: Option<i64>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum DestinationMode {
    Existing,
    New,
}
