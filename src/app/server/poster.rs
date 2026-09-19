use leptos::prelude::ServerFnError;
use std::path::Path;

const ALLOWED_POSTER_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif"];

const MAX_POSTER_BYTES: usize = 5 * 1024 * 1024;

fn validate_poster(ext: &str, bytes: &[u8]) -> Result<(), ServerFnError> {
    let lower = ext.to_ascii_lowercase();
    if !ALLOWED_POSTER_EXTS.contains(&lower.as_str()) {
        return Err(ServerFnError::new(format!("صيغة صورة غير مدعومة: .{ext}")));
    }
    if bytes.len() > MAX_POSTER_BYTES {
        return Err(ServerFnError::new(format!(
            "حجم الصورة يتجاوز {} ميجابايت",
            MAX_POSTER_BYTES / 1024 / 1024
        )));
    }
    Ok(())
}

pub async fn write_poster(
    data_dir: &Path,
    subdir: &str,
    entity_id: i64,
    ext: &str,
    bytes: &[u8],
) -> Result<String, ServerFnError> {
    validate_poster(ext, bytes)?;
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
