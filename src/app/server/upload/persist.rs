use toasty::{Db, Transaction};
use uuid::Uuid;

use super::types::{StagedFile, UploadPayload, UploadTarget};
use crate::app::error::{AppError, ErrorContext};
use crate::app::model::{AudioCollectionRow, Item, VideoCollectionRow};
use crate::app::server::db;

pub async fn insert_files(
    db: &mut Db,
    staged: &[StagedFile],
) -> Result<Vec<(Uuid, String)>, AppError> {
    let mut out = Vec::with_capacity(staged.len());
    for f in staged {
        let id = db::insert_file(db, &f.rel, f.size as i64, f.duration)
            .await
            .context("upload: insert file")?;
        out.push((id, f.title.clone()));
    }
    Ok(out)
}

async fn next_item_number_tx(
    tx: &mut Transaction<'_>,
    collection_id: Uuid,
    season_number: Option<i64>,
) -> toasty::Result<i64> {
    let mut q = Item::filter(Item::fields().collection_id().eq(collection_id));
    q = match season_number {
        Some(s) => q.filter(Item::fields().season_number().eq(s)),
        None => q.filter(Item::fields().season_number().is_none()),
    };
    let last = q
        .order_by(Item::fields().number().desc())
        .limit(1)
        .exec(tx)
        .await?;
    Ok(last.first().map(|i| i.number + 1).unwrap_or(0))
}

static INSERT_LOCK: std::sync::LazyLock<tokio::sync::Mutex<()>> =
    std::sync::LazyLock::new(|| tokio::sync::Mutex::new(()));

async fn bump_count(
    tx: &mut Transaction<'_>,
    collection_id: Uuid,
    delta: i64,
) -> Result<(), AppError> {
    if let Ok(mut c) = VideoCollectionRow::get_by_id(tx, &collection_id).await {
        let n = c.items_count + delta;
        c.update()
            .items_count(n)
            .exec(tx)
            .await
            .context("upload: bump video items_count")?;
        return Ok(());
    }
    let mut c = AudioCollectionRow::get_by_id(tx, &collection_id)
        .await
        .context("upload: load audio collection")?;
    let n = c.items_count + delta;
    c.update()
        .items_count(n)
        .exec(tx)
        .await
        .context("upload: bump audio items_count")?;
    Ok(())
}

pub async fn insert_items_and_poster(
    db: &mut Db,
    payload: &UploadPayload,
    file_rows: &[(Uuid, String)],
) -> Result<(), AppError> {
    let _guard = INSERT_LOCK.lock().await;
    let mut tx = db
        .transaction()
        .await
        .context("upload: begin transaction")?;

    let collection_id = match &payload.target {
        UploadTarget::Existing { collection_id } => *collection_id,
        UploadTarget::New { id, .. } => *id,
    };

    let start = next_item_number_tx(&mut tx, collection_id, payload.season_number)
        .await
        .context("upload: next item number")?;

    for (i, (fid, title)) in file_rows.iter().enumerate() {
        toasty::create!(Item {
            collection_id,
            number: start + i as i64,
            season_number: payload.season_number,
            title: Some(title.clone()),
            poster: None,
            description: None,
            file_id: *fid,
        })
        .exec(&mut tx)
        .await
        .context("upload: insert item")?;
    }

    bump_count(&mut tx, collection_id, file_rows.len() as i64).await?;

    tx.commit().await.context("upload: commit transaction")?;
    Ok(())
}
