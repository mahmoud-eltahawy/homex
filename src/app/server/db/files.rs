use toasty::Db;

use crate::app::model::{File, Item};

// ─── File operations ──────────────────────────────────────────────────────

pub async fn insert_file(
    db: &mut Db,
    relative_path: &str,
    size_bytes: i64,
    duration_secs: i64,
) -> toasty::Result<u64> {
    let f = toasty::create!(File {
        relative_path: relative_path.to_string(),
        size_bytes,
        duration_secs,
    })
    .exec(db)
    .await?;
    Ok(f.id)
}

pub async fn fetch_file_path(db: &mut Db, file_id: i64) -> toasty::Result<Option<String>> {
    Ok(File::get_by_id(db, &(file_id as u64))
        .await
        .ok()
        .map(|f| f.relative_path))
}

// ─── Orphan cleanup ───────────────────────────────────────────────────────

/// For each file id, delete the file row if no item still references it,
/// and return the relative paths so the caller can unlink the media on disk.
pub(crate) async fn purge_orphan_files(
    tx: &mut toasty::Transaction<'_>,
    file_ids: impl IntoIterator<Item = u64>,
) -> toasty::Result<Vec<String>> {
    let mut paths = Vec::new();
    for fid in file_ids {
        let still_used = !Item::filter(Item::fields().file_id().eq(fid))
            .limit(1)
            .exec(tx)
            .await?
            .is_empty();
        if still_used {
            continue;
        }
        if let Ok(f) = File::get_by_id(tx, &fid).await {
            paths.push(f.relative_path.clone());
            f.delete().exec(tx).await?;
        }
    }
    Ok(paths)
}
