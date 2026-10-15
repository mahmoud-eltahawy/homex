use sqlx::SqliteConnection;
use uuid::Uuid;

use crate::app::model::File;

// ─── File operations ──────────────────────────────────────────────────────────

pub async fn insert_file(
    conn: &mut SqliteConnection,
    relative_path: &str,
    size_bytes: i64,
    duration_secs: i64,
) -> sqlx::Result<Uuid> {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO files (id, relative_path, size_bytes, duration_secs)
           VALUES (?, ?, ?, ?)"#,
        id,
        relative_path,
        size_bytes,
        duration_secs,
    )
    .execute(conn)
    .await?;
    Ok(id)
}

pub async fn fetch_file_path(
    pool: &sqlx::SqlitePool,
    file_id: Uuid,
) -> sqlx::Result<Option<String>> {
    sqlx::query_scalar!(
        r#"SELECT relative_path AS "relative_path!: String"
           FROM files WHERE id = ?"#,
        file_id,
    )
    .fetch_optional(pool)
    .await
}

// ─── Orphan cleanup ───────────────────────────────────────────────────────────

pub(crate) async fn purge_orphan_files(
    conn: &mut SqliteConnection,
    file_ids: impl IntoIterator<Item = Uuid>,
) -> sqlx::Result<Vec<String>> {
    let mut paths = Vec::new();
    for fid in file_ids {
        let still_used: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "n!: i64" FROM items WHERE file_id = ? LIMIT 1"#,
            fid,
        )
        .fetch_one(&mut *conn)
        .await?;
        if still_used > 0 {
            continue;
        }
        let row: Option<(Uuid, String, i64, i64)> = sqlx::query_as!(
            File,
            r#"SELECT id AS "id!: Uuid",
                      relative_path AS "relative_path!: String",
                      size_bytes    AS "size_bytes!: i64",
                      duration_secs AS "duration_secs!: i64"
               FROM files WHERE id = ?"#,
            fid,
        )
        .fetch_optional(&mut *conn)
        .await
        .map(|o| o.map(|f| (f.id, f.relative_path, f.size_bytes, f.duration_secs)))?;

        if let Some((_, rel, _, _)) = row {
            paths.push(rel);
            sqlx::query!(r#"DELETE FROM files WHERE id = ?"#, fid)
                .execute(&mut *conn)
                .await?;
        }
    }
    Ok(paths)
}
