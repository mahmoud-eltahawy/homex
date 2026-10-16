use sqlx::SqliteConnection;
use uuid::Uuid;

use crate::app::types::RelPath;

// ─── File operations ──────────────────────────────────────────────────────

pub async fn insert_file(
    conn: &mut SqliteConnection,
    relative_path: &RelPath,
    size_bytes: i64,
    duration_secs: i64,
) -> sqlx::Result<Uuid> {
    let id = Uuid::new_v4();
    sqlx::query!(
        r#"INSERT INTO files (id, relative_path, size_bytes, duration_secs)
           VALUES (?, ?, ?, ?)"#,
        id,
        relative_path.as_str(),
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
) -> sqlx::Result<Option<RelPath>> {
    let raw: Option<String> = sqlx::query_scalar!(
        r#"SELECT relative_path AS "relative_path!: String"
           FROM files WHERE id = ?"#,
        file_id,
    )
    .fetch_optional(pool)
    .await?;
    Ok(raw.map(RelPath::from_stored))
}

// ─── Orphan cleanup ───────────────────────────────────────────────────────

pub(crate) async fn purge_orphan_files(
    conn: &mut SqliteConnection,
    file_ids: impl IntoIterator<Item = Uuid>,
) -> sqlx::Result<Vec<RelPath>> {
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
        let raw: Option<String> = sqlx::query_scalar!(
            r#"SELECT relative_path AS "relative_path!: String"
               FROM files WHERE id = ?"#,
            fid,
        )
        .fetch_optional(&mut *conn)
        .await?;
        if let Some(s) = raw {
            paths.push(RelPath::from_stored(s));
            sqlx::query!(r#"DELETE FROM files WHERE id = ?"#, fid)
                .execute(&mut *conn)
                .await?;
        }
    }
    Ok(paths)
}
