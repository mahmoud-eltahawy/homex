use leptos::prelude::*;

#[cfg(feature = "ssr")]
fn table_for(kind: &str) -> Result<&'static str, ServerFnError> {
    use crate::app::model::MediaType;
    Ok(MediaType::try_from(kind)
        .map_err(|_| ServerFnError::new("نوع غير معروف"))?
        .table())
}

// ─── Create empty item ──────────────────────────────────────────────────────

// ─── Patch a chapter/episode/audio title ────────────────────────────────────

#[server]
pub async fn patch_child_title(kind: String, id: u64, title: String) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    use sqlx::AssertSqlSafe;

    let state: AppState = expect_context();
    let table = match kind.as_str() {
        "movie_chapter" => "movie_chapters",
        "episode" => "episodes",
        "audio" => "audios",
        _ => return Err(ServerFnError::new("نوع غير معروف")),
    };

    let t = if title.trim().is_empty() {
        None
    } else {
        Some(title)
    };

    sqlx::query(AssertSqlSafe(format!(
        "UPDATE {table} SET title = ? WHERE id = ?"
    )))
    .bind(t)
    .bind(id as i64)
    .execute(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(())
}

#[server]
pub async fn create_empty(kind: String) -> Result<u64, ServerFnError> {
    use crate::app::server::AppState;
    use sqlx::AssertSqlSafe;

    let state: AppState = expect_context();
    let table = table_for(&kind)?;

    let id: i64 = sqlx::query_scalar(AssertSqlSafe(format!(
        "INSERT INTO {table} (title) VALUES ('بدون عنوان') RETURNING id"
    )))
    .fetch_one(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(id as u64)
}

// ─── Patch a single scalar field ────────────────────────────────────────────

#[server]
pub async fn patch_field(
    kind: String,
    id: u64,
    field: String,
    value: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    use sqlx::AssertSqlSafe;

    let state: AppState = expect_context();
    let table = table_for(&kind)?;

    let allowed: &[&str] = &["title", "description", "poster"];
    if !allowed.contains(&field.as_str()) {
        return Err(ServerFnError::new("حقل غير مسموح"));
    }

    let q = match field.as_str() {
        "title" => {
            let v = value
                .filter(|s| !s.trim().is_empty())
                .ok_or_else(|| ServerFnError::new("العنوان مطلوب"))?;
            sqlx::query(AssertSqlSafe(format!(
                "UPDATE {table} SET title = ? WHERE id = ?"
            )))
            .bind(v)
            .bind(id as i64)
            .execute(&state.db)
            .await
        }
        "description" => {
            let v = value.filter(|s| !s.trim().is_empty());
            sqlx::query(AssertSqlSafe(format!(
                "UPDATE {table} SET description = ? WHERE id = ?"
            )))
            .bind(v)
            .bind(id as i64)
            .execute(&state.db)
            .await
        }
        "poster" => {
            sqlx::query(AssertSqlSafe(format!(
                "UPDATE {table} SET poster = ? WHERE id = ?"
            )))
            .bind(value)
            .bind(id as i64)
            .execute(&state.db)
            .await
        }
        _ => unreachable!(),
    };

    q.map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ─── Delete item ────────────────────────────────────────────────────────────

#[server]
pub async fn delete_item(kind: String, id: u64) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    use sqlx::AssertSqlSafe;

    let state: AppState = expect_context();
    let table = table_for(&kind)?;

    sqlx::query(AssertSqlSafe(format!("DELETE FROM {table} WHERE id = ?")))
        .bind(id as i64)
        .execute(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ─── Delete a chapter/episode/audio row ─────────────────────────────────────

#[server]
pub async fn delete_child(kind: String, id: u64) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    use sqlx::AssertSqlSafe;

    let state: AppState = expect_context();
    let table = match kind.as_str() {
        "movie_chapter" => "movie_chapters",
        "episode" => "episodes",
        "audio" => "audios",
        _ => return Err(ServerFnError::new("نوع غير معروف")),
    };

    sqlx::query(AssertSqlSafe(format!("DELETE FROM {table} WHERE id = ?")))
        .bind(id as i64)
        .execute(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

// ─── Poster upload (multipart) ──────────────────────────────────────────────

#[server(input = server_fn::codec::MultipartFormData)]
pub async fn upload_poster_inline(
    data: server_fn::codec::MultipartData,
) -> Result<String, ServerFnError> {
    use crate::app::model::MediaType;
    use crate::app::server::{AppState, poster::write_poster, upload::extension_of};
    use sqlx::AssertSqlSafe;

    let state: AppState = expect_context();
    let mut mp = data.into_inner().unwrap();

    let mut kind = String::new();
    let mut id: i64 = 0;
    let mut bytes: Option<(String, Vec<u8>)> = None;

    while let Some(f) = mp.next_field().await? {
        match f.name().unwrap_or("") {
            "kind" => kind = f.text().await?,
            "id" => id = f.text().await?.parse().unwrap_or(0),
            "poster_file" => {
                let fname = f.file_name().unwrap_or("").to_string();
                let b = f.bytes().await?.to_vec();
                if !b.is_empty() {
                    bytes = Some((extension_of(&fname), b));
                }
            }
            _ => {
                let _ = f.bytes().await?;
            }
        }
    }

    let (ext, bytes) = bytes.ok_or_else(|| ServerFnError::new("لم يتم استلام صورة"))?;
    let mt = MediaType::try_from(kind.as_str()).map_err(|_| ServerFnError::new("نوع غير معروف"))?;

    let url = write_poster(
        &state.config.storage.data_dir,
        mt.poster_subdir(),
        id,
        &ext,
        &bytes,
    )
    .await?;

    let table = mt.table();
    sqlx::query(AssertSqlSafe(format!(
        "UPDATE {table} SET poster = ? WHERE id = ?"
    )))
    .bind(&url)
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(url)
}
