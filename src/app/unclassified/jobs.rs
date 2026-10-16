//! Client-side job wrappers for the two classify flows.

use leptos::prelude::*;

use crate::app::error::AppError;
use crate::app::mutations::Mutation;
use crate::app::types::RelPath;

use super::tracker::BatchTracker;
use super::types::{ClassifyDestination, ClassifyInput, FileJobHandle};
use crate::app::unclassified::cancel_classify;
use crate::app::unclassified::{classify_unclassified_files, classify_unclassified_folders};

type ClassifyMutation =
    Mutation<(Vec<ClassifyInput>, ClassifyDestination, Option<i64>), Vec<FileJobHandle>, AppError>;

#[derive(Clone, Copy)]
pub struct ClassifyJob {
    action: ClassifyMutation,
    tracker: BatchTracker,
}

impl ClassifyJob {
    pub fn new() -> Self {
        let tracker = BatchTracker::new();
        let action = Mutation::new(
            |(files, dest, season): &(Vec<ClassifyInput>, ClassifyDestination, Option<i64>)| {
                classify_unclassified_files(files.clone(), dest.clone(), *season)
            },
        )
        .on_success(move |handles| tracker.begin(handles))
        .on_error(move |e| tracker.error_pub().set(Some(e)));

        Self { action, tracker }
    }

    pub fn dispatch(
        &self,
        files: Vec<ClassifyInput>,
        dest: ClassifyDestination,
        season: Option<i64>,
    ) {
        self.tracker.reset();
        self.action.dispatch((files, dest, season));
    }

    pub fn reset(&self) {
        self.tracker.reset();
    }

    pub fn tracker(&self) -> BatchTracker {
        self.tracker
    }

    pub fn cancel(&self) {
        let ids = self.tracker.pending_job_ids();
        if ids.is_empty() {
            return;
        }
        leptos::task::spawn_local(async move {
            let _ = cancel_classify(ids).await;
        });
    }
}

#[derive(Clone, Copy)]
pub struct FolderClassifyJob {
    action: Mutation<(Vec<RelPath>, ClassifyDestination), Vec<FileJobHandle>, AppError>,
    tracker: BatchTracker,
}

impl FolderClassifyJob {
    pub fn new() -> Self {
        let tracker = BatchTracker::new();
        let action = Mutation::new(|(paths, dest): &(Vec<RelPath>, ClassifyDestination)| {
            classify_unclassified_folders(paths.clone(), dest.clone())
        })
        .on_success(move |handles| tracker.begin(handles))
        .on_error(move |e| tracker.error_pub().set(Some(e)));

        Self { action, tracker }
    }

    pub fn dispatch(&self, paths: Vec<RelPath>, dest: ClassifyDestination) {
        self.tracker.reset();
        self.action.dispatch((paths, dest));
    }

    pub fn reset(&self) {
        self.tracker.reset();
    }

    pub fn tracker(&self) -> BatchTracker {
        self.tracker
    }

    pub fn cancel(&self) {
        let ids = self.tracker.pending_job_ids();
        if ids.is_empty() {
            return;
        }
        leptos::task::spawn_local(async move {
            let _ = cancel_classify(ids).await;
        });
    }
}
