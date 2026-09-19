use leptos::prelude::ServerFnError;
use std::path::Path;

pub async fn write_poster(
    data_dir: &Path,
    subdir: &str,
    entity_id: i64,
    ext: &str,
    bytes: &[u8],
) -> Result<String, ServerFnError> {
    let dir = data_dir.join("posters").join(subdir);
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| ServerFnError::new(format!("mkdir posters: {e}")))?;

    let filename = format!("{entity_id}.{ext}");
    tokio::fs::write(dir.join(&filename), bytes)
        .await
        .map_err(|e| ServerFnError::new(format!("write poster: {e}")))?;

    Ok(format!("/posters/{subdir}/{filename}"))
}
