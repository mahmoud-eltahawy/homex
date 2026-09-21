use crate::app::model::{Collection, Item, MediaFile};
use leptos::prelude::*;

#[server]
pub async fn fetch_collections(
    section_slug: String,
    offset: usize,
    size: usize,
    search_query: Option<String>,
) -> Result<Vec<Collection>, ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();

    #[derive(sqlx::FromRow)]
    struct Row {
        id: i64,
        section_id: i64,
        section_slug: String,
        title: String,
        poster: Option<String>,
        description: Option<String>,
        items_count: i64,
    }
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT c.id, c.section_id, s.slug AS section_slug, \
                c.title, c.poster, c.description, \
                (SELECT COUNT(*) FROM items i WHERE i.collection_id = c.id) AS items_count \
         FROM collections c JOIN sections s ON s.id = c.section_id \
         WHERE s.slug = ?1 \
           AND (?2 IS NULL OR c.title LIKE '%' || ?2 || '%') \
         ORDER BY c.position, c.title LIMIT ?3 OFFSET ?4",
    )
    .bind(&section_slug)
    .bind(&search_query)
    .bind(size as i64)
    .bind(offset as i64)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|r| Collection {
            id: r.id as u64,
            section_id: r.section_id as u64,
            section_slug: r.section_slug,
            title: r.title,
            poster: r.poster,
            description: r.description,
            items_count: r.items_count as u32,
        })
        .collect())
}

#[server]
pub async fn fetch_collections_count(
    section_slug: String,
    search_query: Option<String>,
) -> Result<usize, ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM collections c JOIN sections s ON s.id = c.section_id \
         WHERE s.slug = ?1 AND (?2 IS NULL OR c.title LIKE '%' || ?2 || '%')",
    )
    .bind(&section_slug)
    .bind(&search_query)
    .fetch_one(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(n as usize)
}

#[server]
pub async fn fetch_collection_detail(
    section_slug: String,
    collection_id: u64,
) -> Result<Collection, ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();

    #[derive(sqlx::FromRow)]
    struct Row {
        id: i64,
        section_id: i64,
        section_slug: String,
        title: String,
        poster: Option<String>,
        description: Option<String>,
        items_count: i64,
    }
    let r: Row = sqlx::query_as(
        "SELECT c.id, c.section_id, s.slug AS section_slug, \
                c.title, c.poster, c.description, \
                (SELECT COUNT(*) FROM items i WHERE i.collection_id = c.id) AS items_count \
         FROM collections c JOIN sections s ON s.id = c.section_id \
         WHERE s.slug = ? AND c.id = ?",
    )
    .bind(&section_slug)
    .bind(collection_id as i64)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?
    .ok_or_else(|| ServerFnError::new("collection not found"))?;

    Ok(Collection {
        id: r.id as u64,
        section_id: r.section_id as u64,
        section_slug: r.section_slug,
        title: r.title,
        poster: r.poster,
        description: r.description,
        items_count: r.items_count as u32,
    })
}

#[server]
pub async fn fetch_items(collection_id: u64) -> Result<Vec<Item>, ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();

    #[derive(sqlx::FromRow)]
    struct Row {
        id: i64,
        collection_id: i64,
        number: i64,
        season_number: Option<i64>,
        title: Option<String>,
        poster: Option<String>,
        description: Option<String>,
        file_id: i64,
        size: i64,
        dur: i64,
    }
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT i.id, i.collection_id, i.number, i.season_number, \
                i.title, i.poster, i.description, \
                f.id AS file_id, f.size_bytes AS size, f.duration_secs AS dur \
         FROM items i JOIN files f ON f.id = i.file_id \
         WHERE i.collection_id = ? \
         ORDER BY COALESCE(i.season_number, 0), i.number",
    )
    .bind(collection_id as i64)
    .fetch_all(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(rows
        .into_iter()
        .map(|r| Item {
            id: r.id as u64,
            collection_id: r.collection_id as u64,
            number: r.number,
            season_number: r.season_number,
            title: r.title,
            poster: r.poster,
            description: r.description,
            file: MediaFile {
                id: r.file_id as u64,
                path: format!("/media/{}", r.file_id),
                size: r.size as u64,
                duration: r.dur as u64,
            },
        })
        .collect())
}

#[server]
pub async fn create_empty_collection(section_slug: String) -> Result<u64, ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    let section_id: i64 = sqlx::query_scalar("SELECT id FROM sections WHERE slug = ?")
        .bind(&section_slug)
        .fetch_one(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    let id: i64 = sqlx::query_scalar(
        "INSERT INTO collections (section_id, title, position) \
         VALUES (?, 'بدون عنوان', \
                 (SELECT COALESCE(MAX(position)+1,0) FROM collections WHERE section_id=?)) \
         RETURNING id",
    )
    .bind(section_id)
    .bind(section_id)
    .fetch_one(&state.db)
    .await
    .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(id as u64)
}

#[server]
pub async fn patch_collection_field(
    id: u64,
    field: String,
    value: Option<String>,
) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    use sqlx::AssertSqlSafe;
    let state: AppState = expect_context();
    let col = match field.as_str() {
        "title" | "description" | "poster" => field,
        _ => return Err(ServerFnError::new("حقل غير مسموح")),
    };
    let sql = AssertSqlSafe(format!("UPDATE collections SET {col} = ? WHERE id = ?"));
    sqlx::query(sql)
        .bind(value)
        .bind(id as i64)
        .execute(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[server]
pub async fn patch_item_title(id: u64, title: String) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    let t = if title.trim().is_empty() {
        None
    } else {
        Some(title)
    };
    sqlx::query("UPDATE items SET title = ? WHERE id = ?")
        .bind(t)
        .bind(id as i64)
        .execute(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[server]
pub async fn delete_collection(id: u64) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    sqlx::query("DELETE FROM collections WHERE id=?")
        .bind(id as i64)
        .execute(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[server]
pub async fn delete_item(id: u64) -> Result<(), ServerFnError> {
    use crate::app::server::AppState;
    let state: AppState = expect_context();
    sqlx::query("DELETE FROM items WHERE id=?")
        .bind(id as i64)
        .execute(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(())
}

#[server(input = server_fn::codec::MultipartFormData)]
pub async fn upload_collection_poster(
    data: server_fn::codec::MultipartData,
) -> Result<String, ServerFnError> {
    use crate::app::server::{AppState, poster::write_poster, upload::extension_of};
    use sqlx::AssertSqlSafe;

    let state: AppState = expect_context();
    let mut mp = data.into_inner().unwrap();
    let mut section_slug = String::new();
    let mut id: i64 = 0;
    let mut bytes: Option<(String, Vec<u8>)> = None;

    while let Some(f) = mp.next_field().await? {
        match f.name().unwrap_or("") {
            "section" => section_slug = f.text().await?,
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
    let url = write_poster(
        &state.config.storage.data_dir,
        &section_slug,
        id,
        &ext,
        &bytes,
    )
    .await?;

    let sql = AssertSqlSafe("UPDATE collections SET poster = ? WHERE id = ?".to_string());
    sqlx::query(sql)
        .bind(&url)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(url)
}
