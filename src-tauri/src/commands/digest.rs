use crate::structs::Progress;
use sha2::{Digest, Sha256};
use tauri::Emitter;

#[tauri::command]
pub async fn sha256_digest(app: tauri::AppHandle, chunk: Vec<u8>, id: String) -> Result<Vec<u8>, String> {
    let _ = app.emit(
        "progress",
        Progress {
            r#type: "completed".into(),
            stage: "hashing".into(),
            progress: None,
            id: id.clone(),
        },
    );

    let mut hasher = Sha256::new();
    hasher.update(&chunk);

    Ok(hasher.finalize().to_vec())
}
