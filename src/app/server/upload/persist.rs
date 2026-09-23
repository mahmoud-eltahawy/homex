use crate::app::constants::messages;
use leptos::prelude::ServerFnError;
use std::path::Path;
use toasty::{Db, Transaction};

use super::types::{StagedFile, UploadPayload};
use crate::app::model::{Collection, Item};
use crate::app::server::db;
use crate::app::server::poster::write_poster;

fn toasty_err(e: toasty::Error) -> ServerFnError {
    leptos::logging::error!("[upload] {e}");
    ServerFnError::new(messages::INTERNAL_ERROR)
}

pub async fn insert_files(
    db: &mut Db,
    staged: &[StagedFile],
) -> Result<Vec<(u64, String)>, ServerFnError> {
    let mut out = Vec::with_capacity(staged.len());
    for f in staged {
        let id = db::insert_file(db, &f.rel, f.size as i64, f.duration)
            .await
            .map_err(toasty_err)?;
        out.push((id, f.title.clone()));
    }
    Ok(out)
}

async fn next_item_number_tx(
    tx: &mut Transaction<'_>,
    collection_id: u64,
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

pub async fn insert_items_and_poster(
    db: &mut Db,
    payload: &UploadPayload,
    file_rows: &[(u64, String)],
    data_dir: &Path,
) -> Result<(), ServerFnError> {
    let mut tx = db.transaction().await.map_err(toasty_err)?;

    let collection_id = payload.collection_id as u64;
    let start = next_item_number_tx(&mut tx, collection_id, payload.season_number)
        .await
        .map_err(toasty_err)?;

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
        .map_err(toasty_err)?;
    }

    let mut collection = Collection::get_by_id(&mut tx, &collection_id)
        .await
        .map_err(toasty_err)?;
    let new_count = collection.items_count + file_rows.len() as i64;
    collection
        .update()
        .items_count(new_count)
        .exec(&mut tx)
        .await
        .map_err(toasty_err)?;

    if let Some((ext, bytes)) = payload.poster.as_ref() {
        let url = write_poster(
            data_dir,
            &payload.section_slug,
            collection_id as i64,
            ext,
            bytes,
        )
        .await?;
        collection
            .update()
            .poster(Some(url))
            .exec(&mut tx)
            .await
            .map_err(toasty_err)?;
    }

    tx.commit().await.map_err(toasty_err)?;
    Ok(())
}
