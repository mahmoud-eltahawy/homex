use leptos::prelude::ServerFnError;
use sqlx::SqlitePool;
use std::path::Path;

use super::types::{StagedFile, UploadPayload};
use crate::app::server::SqlErr;
use crate::app::server::db;
use crate::app::server::poster::write_poster;

pub async fn insert_files(
    pool: &SqlitePool,
    staged: &[StagedFile],
) -> Result<Vec<(i64, String)>, ServerFnError> {
    let mut out = Vec::with_capacity(staged.len());
    for f in staged {
        let id = db::insert_file(pool, &f.rel, f.size as i64, f.duration)
            .await
            .srv()?;
        out.push((id, f.title.clone()));
    }
    Ok(out)
}

pub async fn insert_items(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    collection_id: i64,
    season_number: Option<i64>,
    file_rows: &[(i64, String)],
) -> Result<(), ServerFnError> {
    let start = db::next_item_number(&mut **tx, collection_id, season_number)
        .await
        .srv()?;

    for (i, (fid, title)) in file_rows.iter().enumerate() {
        db::insert_item(
            &mut **tx,
            collection_id,
            start + i as i64,
            season_number,
            Some(title.as_str()),
            *fid,
        )
        .await
        .srv()?;
    }
    Ok(())
}

pub async fn attach_poster(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    payload: &UploadPayload,
    collection_id: i64,
    data_dir: &Path,
) -> Result<(), ServerFnError> {
    let Some((ext, bytes)) = payload.poster.as_ref() else {
        return Ok(());
    };
    let url = write_poster(data_dir, &payload.section_slug, collection_id, ext, bytes).await?;
    db::update_collection_poster(&mut **tx, collection_id, Some(&url))
        .await
        .srv()?;
    Ok(())
}
