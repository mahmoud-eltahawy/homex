use sqlx::{SqliteConnection, SqlitePool};
use uuid::Uuid;

use crate::app::model::{Tag, canonicalize_tags};

// ─── Tag loaders / writers ────────────────────────────────────────────────────

pub async fn fetch_all_tags(pool: &SqlitePool) -> sqlx::Result<Vec<Tag>> {
    sqlx::query_as!(
        Tag,
        r#"SELECT id AS "id!: Uuid", name AS "name!: String"
           FROM tags ORDER BY name ASC"#,
    )
    .fetch_all(pool)
    .await
}

async fn find_tag_id_by_name(pool: &SqlitePool, name: &str) -> sqlx::Result<Option<Uuid>> {
    sqlx::query_scalar!(
        r#"SELECT id AS "id!: Uuid" FROM tags WHERE name = ? COLLATE NOCASE"#,
        name,
    )
    .fetch_optional(pool)
    .await
}

async fn upsert_tag_tx(conn: &mut SqliteConnection, name: &str) -> sqlx::Result<Tag> {
    if let Some(t) = sqlx::query_as!(
        Tag,
        r#"SELECT id AS "id!: Uuid", name AS "name!: String"
           FROM tags WHERE name = ?"#,
        name,
    )
    .fetch_optional(&mut *conn)
    .await?
    {
        return Ok(t);
    }
    let id = Uuid::new_v4();
    sqlx::query!(r#"INSERT INTO tags (id, name) VALUES (?, ?)"#, id, name)
        .execute(&mut *conn)
        .await?;
    Ok(Tag {
        id,
        name: name.to_string(),
    })
}

async fn tags_for_joins(pool: &SqlitePool, tag_ids: &[Uuid]) -> sqlx::Result<Vec<Tag>> {
    let mut out = Vec::with_capacity(tag_ids.len());
    for id in tag_ids {
        let row = sqlx::query_as!(
            Tag,
            r#"SELECT id AS "id!: Uuid", name AS "name!: String"
               FROM tags WHERE id = ?"#,
            id,
        )
        .fetch_optional(pool)
        .await?;
        match row {
            Some(t) => out.push(t),
            None => leptos::logging::warn!("[tags] dangling tag_id {id}"),
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub async fn fetch_section_tags(pool: &SqlitePool, section_id: Uuid) -> sqlx::Result<Vec<Tag>> {
    let ids: Vec<Uuid> = sqlx::query_scalar!(
        r#"SELECT tag_id AS "tag_id!: Uuid"
           FROM section_tags WHERE section_id = ?"#,
        section_id,
    )
    .fetch_all(pool)
    .await?;
    tags_for_joins(pool, &ids).await
}

pub async fn fetch_collection_tags(
    pool: &SqlitePool,
    collection_id: Uuid,
) -> sqlx::Result<Vec<Tag>> {
    let ids: Vec<Uuid> = sqlx::query_scalar!(
        r#"SELECT tag_id AS "tag_id!: Uuid"
           FROM collection_tags WHERE collection_id = ?"#,
        collection_id,
    )
    .fetch_all(pool)
    .await?;
    tags_for_joins(pool, &ids).await
}

pub async fn set_section_tags(
    pool: &SqlitePool,
    section_id: Uuid,
    names: &[String],
) -> sqlx::Result<()> {
    let normalized = canonicalize_tags(names);
    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"DELETE FROM section_tags WHERE section_id = ?"#,
        section_id
    )
    .execute(&mut *tx)
    .await?;

    for name in &normalized {
        let tag = upsert_tag_tx(&mut tx, name).await?;
        sqlx::query!(
            r#"INSERT INTO section_tags (id, section_id, tag_id) VALUES (?, ?, ?)"#,
            Uuid::new_v4(),
            section_id,
            tag.id,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

pub async fn set_collection_tags(
    pool: &SqlitePool,
    collection_id: Uuid,
    names: &[String],
) -> sqlx::Result<()> {
    let normalized = canonicalize_tags(names);
    let mut tx = pool.begin().await?;

    sqlx::query!(
        r#"DELETE FROM collection_tags WHERE collection_id = ?"#,
        collection_id
    )
    .execute(&mut *tx)
    .await?;

    for name in &normalized {
        let tag = upsert_tag_tx(&mut tx, name).await?;
        sqlx::query!(
            r#"INSERT INTO collection_tags (id, collection_id, tag_id) VALUES (?, ?, ?)"#,
            Uuid::new_v4(),
            collection_id,
            tag.id,
        )
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(())
}

// ─── Tag-driven collection filtering ──────────────────────────────────────────

pub(crate) async fn collection_ids_with_all_tags(
    pool: &SqlitePool,
    tag_names: &[String],
) -> sqlx::Result<Vec<Uuid>> {
    let mut intersection: Option<Vec<Uuid>> = None;

    for name in tag_names {
        let Some(tag_id) = find_tag_id_by_name(pool, name).await? else {
            return Ok(Vec::new());
        };
        let ids: Vec<Uuid> = sqlx::query_scalar!(
            r#"SELECT collection_id AS "collection_id!: Uuid"
               FROM collection_tags WHERE tag_id = ?"#,
            tag_id,
        )
        .fetch_all(pool)
        .await?;

        intersection = Some(match intersection {
            None => ids,
            Some(prev) => prev.into_iter().filter(|id| ids.contains(id)).collect(),
        });
        if intersection.as_ref().is_some_and(Vec::is_empty) {
            return Ok(Vec::new());
        }
    }

    let mut ids = intersection.unwrap_or_default();
    ids.sort_unstable();
    ids.dedup();
    Ok(ids)
}
