use std::{fs, str::FromStr};

use crate::{
    process::video::getVideoStats,
    structs::{PickedFile, Video},
};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_fs::FilePath::{self, Url};

fn genericPickFile(app: &tauri::AppHandle, path: Option<String>) -> Result<PickedFile, String> {
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
            name,
        })
    } else {
        return Err("No file selected".into());
    }
}

#[cfg(target_os = "android")]
fn androidPickFile(app: tauri::AppHandle, id: String) -> Result<PickedFile, String> {
    use std::io::Write;
    use tauri::Emitter;
    use tauri_plugin_fs::FsExt;
    use crate::structs::Progress;

    let originalFile = genericPickFile(&app, None)?;
    let _ = app.emit(
        "progress",
        Progress {
            r#type: "completed".into(),
            stage: "copying".into(),
            progress: None,
            id,
        },
    );

    let file = app
        .fs()
        .read(originalFile.path.clone())
        .map_err(|_| "Failed to read file.")?;

    let targetPath = app.path().app_cache_dir().unwrap().join(&originalFile.name);
    let mut targetFile = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(targetPath.clone())
        .map_err(|_| "Failed to create file copy for processing.")?;

    targetFile
        .write_all(&file)
        .map_err(|_| "Failed to write to copy file for processing.")?;
    targetFile
        .flush()
        .map_err(|_| "Failed to flush copy file for processing.")?;

    Ok(PickedFile {
        path: targetPath.into(),
        name: originalFile.name,
    })
}

#[tauri::command]
pub async fn pick_file(app: tauri::AppHandle, id: String, path: Option<String>) -> Result<Video, String> {
    println!("Picking file");

    #[cfg(target_os = "android")]
    {
        let pickedFile = androidPickFile(app, id.clone())?;
        let videoStats = getVideoStats(pickedFile.path.to_string())?;

        return Ok(Video {
            path: pickedFile.path,
            name: pickedFile.name,
            id: id,
            stats: videoStats,
            crf: Some(23),
        });
    }

    #[cfg(not(target_os = "android"))]
    {
        let pickedFile = genericPickFile(&app, path)?;
        let videoStats = getVideoStats(pickedFile.path.to_string())?;

        return Ok(Video {
            path: pickedFile.path,
            name: pickedFile.name,
            id: id,
            stats: videoStats,
            crf: Some(23),
        });
    }
}

#[tauri::command]
pub async fn remove_file(path: FilePath) -> Result<(), String> {
    fs::remove_file(path.into_path().unwrap()).map_err(|e| e.to_string())?;

    Ok(())
}
