use std::io::Write;
use std::{fs::{self, File}, str::FromStr};

use crate::{
    process::video::getVideoStats,
    structs::{FileReaders, PickedFile, Video, VideoRegistry},
};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use tauri::{Manager, State};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_fs::FilePath::{self, Url};

fn genericPickFile(
    app: &tauri::AppHandle,
    path: Option<String>,
) -> Result<PickedFile, String> {
    let fPath = if path.is_some() {
        Some(FilePath::from_str(&path.unwrap()).unwrap())
    } else {
        app.dialog().file().blocking_pick_file()
    };
    if let Some(filePath) = fPath {
        let resolver = app.path();
        let name = match filePath {
            Url(ref url) => resolver.file_name(url.as_str()).unwrap(),
            _ => resolver
                .file_name(filePath.as_path().unwrap().to_str().unwrap())
                .unwrap(),
        };

        Ok(PickedFile {
            path: filePath,
            id: None,
            name,
        })
    } else {
        return Err("No file selected".into());
    }
}

#[tauri::command]
pub async fn pick_file(
    app: tauri::AppHandle,
    reg: State<'_, VideoRegistry>,
    id: String,
    path: Option<String>,
) -> Result<Video, String> {
    let pickedFile = genericPickFile(&app, path)?;
    let videoStats = getVideoStats(pickedFile.path.to_string())?;

    reg.0.lock().unwrap().insert(id.clone(), pickedFile.path.clone());

    return Ok(Video {
        path: pickedFile.path,
        name: pickedFile.name,
        id: id,
        stats: videoStats,
        crf: Some(23),
        cuts: None,
    });
}

#[tauri::command]
pub async fn remove_file(path: FilePath) -> Result<(), String> {
    fs::remove_file(path.into_path().unwrap()).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn write_to_file(
    app: tauri::AppHandle,
    code: String,
    name: String,
    chunk_b64: String,
    eof: bool,
    state: State<'_, FileReaders>,
) -> Result<String, String> {
    let mut filePath = app.path().download_dir().map_err(|e| e.to_string())?;
    filePath.push(&code);
    fs::create_dir_all(&filePath).map_err(|e| e.to_string())?;
    filePath.push(&name);

    let pathStr = filePath.to_string_lossy().to_string();
    let mut readers = state.0.lock().map_err(|_| "State lock failed".to_string())?;

    if !readers.contains_key(&pathStr) {
        let file = File::create(&filePath).map_err(|e| e.to_string())?;
        readers.insert(pathStr.clone(), file);
    }

    if let Some(file) = readers.get_mut(&pathStr) {
        if !chunk_b64.is_empty() {
            let data = STANDARD.decode(&chunk_b64).map_err(|e| e.to_string())?;
            file.write_all(&data).map_err(|e| e.to_string())?;
        }
    }

    if eof {
        readers.remove(&pathStr);
    }

    Ok(pathStr)
}
