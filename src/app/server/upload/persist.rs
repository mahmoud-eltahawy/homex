use super::files::base_subdir;
use super::types::{StagedFile, UploadPayload};
use crate::app::model::MediaType;
use crate::app::server::poster::write_poster;
use leptos::prelude::ServerFnError;
use sqlx::{AssertSqlSafe, SqlitePool};
use std::path::Path;

/// Insert one `files` row per staged file, returning `(file_id, title)` pairs
/// in the same order the files appear in the payload.
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

pub async fn persist_movie(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    payload: &UploadPayload,
    file_rows: &[(i64, String)],
) -> Result<i64, ServerFnError> {
    let movie_id: i64 = if payload.is_new {
        sqlx::query_scalar("INSERT INTO movies (title, description) VALUES (?, ?) RETURNING id")
            .bind(&payload.title)
            .bind(non_empty(&payload.description))
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
    } else {
        payload
            .existing_id
            .ok_or_else(|| ServerFnError::new("existing_id required"))?
    };

    let start: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(number) + 1, 0) FROM movie_chapters WHERE movie_id = ?",
    )
    .bind(movie_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    for (i, (fid, title)) in file_rows.iter().enumerate() {
        sqlx::query(
            "INSERT INTO movie_chapters (movie_id, number, title, file_id) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(movie_id)
        .bind(start + i as i64)
        .bind(title)
        .bind(fid)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    }

    Ok(movie_id)
}

pub async fn persist_series(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    payload: &UploadPayload,
    file_rows: &[(i64, String)],
) -> Result<i64, ServerFnError> {
    let series_id: i64 = if payload.is_new {
        sqlx::query_scalar("INSERT INTO series (title, description) VALUES (?, ?) RETURNING id")
            .bind(&payload.title)
            .bind(non_empty(&payload.description))
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?
    } else {
        payload
            .existing_id
            .ok_or_else(|| ServerFnError::new("existing_id required"))?
    };

    let sn = payload.season_number.unwrap_or(1);
    let season_id: i64 = sqlx::query_scalar(
        "INSERT INTO seasons (series_id, number) VALUES (?, ?) \
         ON CONFLICT(series_id, number) DO UPDATE SET number = number \
         RETURNING id",
    )
    .bind(series_id)
    .bind(sn)
    .fetch_one(&mut **tx)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    let start: i64 =
        sqlx::query_scalar("SELECT COALESCE(MAX(number) + 1, 1) FROM episodes WHERE season_id = ?")
            .bind(season_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

    for (i, (fid, title)) in file_rows.iter().enumerate() {
        sqlx::query(
            "INSERT INTO episodes (season_id, number, title, file_id) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(season_id)
        .bind(start + i as i64)
        .bind(title)
        .bind(fid)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    }

    Ok(series_id)
}

pub async fn persist_audio_group(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    payload: &UploadPayload,
    file_rows: &[(i64, String)],
) -> Result<i64, ServerFnError> {
    let group_id: i64 = if payload.is_new {
        sqlx::query_scalar(
            "INSERT INTO audio_groups (title, description) VALUES (?, ?) RETURNING id",
        )
        .bind(&payload.title)
        .bind(non_empty(&payload.description))
        .fetch_one(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?
    } else {
        payload
            .existing_id
            .ok_or_else(|| ServerFnError::new("existing_id required"))?
    };

    let start: i64 =
        sqlx::query_scalar("SELECT COALESCE(MAX(number) + 1, 0) FROM audios WHERE group_id = ?")
            .bind(group_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| ServerFnError::new(e.to_string()))?;

    for (i, (fid, title)) in file_rows.iter().enumerate() {
        sqlx::query(
            "INSERT INTO audios (group_id, number, title, file_id) \
             VALUES (?, ?, ?, ?)",
        )
        .bind(group_id)
        .bind(start + i as i64)
        .bind(title)
        .bind(fid)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    }

    Ok(group_id)
}

pub async fn attach_poster(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    payload: &UploadPayload,
    entity_id: i64,
    data_dir: &Path,
) -> Result<(), ServerFnError> {
    let Some((ext, bytes)) = payload.poster.as_ref() else {
        return Ok(());
    };

    let subdir = base_subdir(payload.media_type);
    let table = match payload.media_type {
        MediaType::Movie => "movies",
        MediaType::Series => "series",
        MediaType::AudioGroup => "audio_groups",
    };

    let url = write_poster(data_dir, subdir, entity_id, ext, bytes).await?;

    let sql = AssertSqlSafe(format!("UPDATE {table} SET poster = ? WHERE id = ?"));
    sqlx::query(sql)
        .bind(&url)
        .bind(entity_id)
        .execute(&mut **tx)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

fn non_empty(s: &str) -> Option<&str> {
    if s.is_empty() { None } else { Some(s) }
}
