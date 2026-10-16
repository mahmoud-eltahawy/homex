use sqlx::SqlitePool;
use uuid::Uuid;

use super::types::{StagedFile, UploadPayload, UploadTarget};
use crate::app::error::{AppError, ErrorContext};
use crate::app::server::db;
use crate::app::types::Title;

pub async fn insert_files(
    pool: &SqlitePool,
    staged: &[StagedFile],
) -> Result<Vec<(Uuid, Title)>, AppError> {
    let mut conn = pool.acquire().await.context("upload: acquire conn")?;
    let mut out = Vec::with_capacity(staged.len());
    for f in staged {
        let id = db::insert_file(&mut conn, &f.rel, f.size as i64, f.duration)
            .await
            .context("upload: insert file")?;
        out.push((id, f.title.clone()));
    }
    Ok(out)
}

async fn next_item_number(
    conn: &mut sqlx::SqliteConnection,
    collection_id: Uuid,
    season_number: Option<i64>,
) -> sqlx::Result<i64> {
    match season_number {
        Some(s) => {
            sqlx::query_scalar!(
                r#"SELECT COALESCE(MAX(number), -1) + 1 AS "n!: i64"
               FROM items WHERE collection_id = ? AND season_number = ?"#,
                collection_id,
                s,
            )
            .fetch_one(&mut *conn)
            .await
        }
        None => {
            sqlx::query_scalar!(
                r#"SELECT COALESCE(MAX(number), -1) + 1 AS "n!: i64"
               FROM items WHERE collection_id = ? AND season_number IS NULL"#,
                collection_id,
            )
            .fetch_one(&mut *conn)
            .await
        }
    }
}

static INSERT_LOCK: std::sync::LazyLock<tokio::sync::Mutex<()>> =
    std::sync::LazyLock::new(|| tokio::sync::Mutex::new(()));

async fn bump_count(
    conn: &mut sqlx::SqliteConnection,
    collection_id: Uuid,
    delta: i64,
) -> Result<(), AppError> {
    let video_hit: i64 = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "n!: i64" FROM video_collections WHERE id = ?"#,
        collection_id,
    )
    .fetch_one(&mut *conn)
    .await
    .context("upload: video probe")?;

    if video_hit > 0 {
        sqlx::query!(
            r#"UPDATE video_collections SET items_count = MAX(items_count + ?, 0) WHERE id = ?"#,
            delta,
            collection_id,
        )
        .execute(&mut *conn)
        .await
        .context("upload: bump video items_count")?;
        return Ok(());
    }
    sqlx::query!(
        r#"UPDATE audio_collections SET items_count = MAX(items_count + ?, 0) WHERE id = ?"#,
        delta,
        collection_id,
    )
    .execute(&mut *conn)
    .await
    .context("upload: bump audio items_count")?;
    Ok(())
}

pub async fn insert_items_and_poster(
    pool: &SqlitePool,
    payload: &UploadPayload,
    file_rows: &[(Uuid, Title)],
) -> Result<(), AppError> {
    let _guard = INSERT_LOCK.lock().await;
    let mut tx = pool.begin().await.context("upload: begin transaction")?;

    let collection_id = match &payload.target {
        UploadTarget::Existing { collection_id } => *collection_id,
        UploadTarget::New { id, .. } => *id,
    };

    let start = next_item_number(&mut tx, collection_id, payload.season_number)
        .await
        .context("upload: next item number")?;

    for (i, (fid, title)) in file_rows.iter().enumerate() {
        sqlx::query!(
            r#"INSERT INTO items
               (id, collection_id, number, season_number, title, poster, description, file_id)
               VALUES (?, ?, ?, ?, ?, NULL, NULL, ?)"#,
            Uuid::new_v4(),
            collection_id,
            start + i as i64,
            payload.season_number,
            title.as_str(),
            fid,
        )
        .execute(&mut *tx)
        .await
        .context("upload: insert item")?;
    }

    bump_count(&mut tx, collection_id, file_rows.len() as i64).await?;

    tx.commit().await.context("upload: commit transaction")?;
    Ok(())
}
