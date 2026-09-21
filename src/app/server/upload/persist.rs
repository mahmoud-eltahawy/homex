use leptos::prelude::ServerFnError;
use sqlx::SqlitePool;
use std::path::Path;

use super::types::{StagedFile, UploadPayload};
use crate::app::server::poster::write_poster;

pub async fn insert_files(
    db: &SqlitePool,
    staged: &[StagedFile],
) -> Result<Vec<(i64, String)>, ServerFnError> {
    let mut out = Vec::with_capacity(staged.len());
    for f in staged {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO files (relative_path, size_bytes, duration_secs) \
             VALUES (?, ?, ?) RETURNING id",
        )
        .bind(&f.rel)
        .bind(f.size as i64)
        .bind(f.duration)
        .fetch_one(db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
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
    let start: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(number)+1, 0) FROM items \
         WHERE collection_id = ?1 AND \
         ((?2 IS NULL AND season_number IS NULL) OR season_number = ?2)",
    )
    .bind(collection_id)
    .bind(season_number)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    for (i, (fid, title)) in file_rows.iter().enumerate() {
        sqlx::query(
            "INSERT INTO items (collection_id, number, season_number, title, file_id) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(collection_id)
        .bind(start + i as i64)
        .bind(season_number)
        .bind(title)
        .bind(fid)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
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
    sqlx::query("UPDATE collections SET poster = ? WHERE id = ?")
        .bind(&url)
        .bind(collection_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}
