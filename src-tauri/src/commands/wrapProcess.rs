use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
};

use tauri::{Emitter, Manager, Runtime, State};
use tauri_plugin_fs::FilePath;

use crate::{
    process::video::processVideo, structs::{Notification, Progress, Video, VideoRegistry},
};

#[tauri::command]
pub async fn process_video<R: Runtime>(app: tauri::AppHandle<R>, reg: State<'_, VideoRegistry>, video: Video) -> Result<(String, String), String> {
    println!("Processing with video info: {:#?}", video);
    let outputFilePath = PathBuf::from(&video.name);
    let outputPath = format!(
        "{}/videoOutput/{}",
        app.path()
            .app_cache_dir()
            .unwrap()
            .as_mut_os_str()
            .to_str()
            .unwrap()
            .to_owned(),
        outputFilePath.with_extension("mp4").to_str().to_owned().unwrap(),
    );
    create_dir_all(Path::new(&outputPath).parent().unwrap()).map_err(|e| e.to_string())?;

    let _ = app.emit(
        "progress",
        Progress {
            r#type: "progress".into(),
            stage: "processing".into(),
            progress: Some(0.0),
            id: video.id.clone(),
        },
    );
    let result = processVideo(&video, outputPath.clone(), |p| {
        let _ = app.emit(
            "progress",
            Progress {
                r#type: "progress".into(),
                stage: "processing".into(),
                progress: Some(p),
                id: video.id.clone(),
            },
        );
    });

    let _ = app.emit(
        "notify",
        if result.is_err() {
            Notification {
                title: "Error".into(),
                message: "Failed to process video file.".into(),
            }
        } else {
            Notification {
                title: "Success".into(),
                message: "Video file processed successfully. Preparing to upload.".into(),
            }
        },
    );

    #[cfg(target_os = "android")] // other os operate on the actual video, not a copy, so those should not have this video deleted
    let _ = std::fs::remove_file(video.path.into_path().unwrap()).is_ok();

    if result.is_err() {
        println!("{}", result.err().unwrap());
        return Err("Failed to process video file".into());
    }

    let _ = app.emit(
        "progress",
        Progress {
            r#type: "progress".into(),
            stage: "processing".into(),
            progress: Some(1.0),
            id: video.id.clone(),
        },
    );

    let id = uuid::Uuid::new_v4().to_string();
    reg.0.lock().unwrap().insert(id.clone(), FilePath::Path(PathBuf::from(outputPath.clone())));
    return Ok((id, outputPath));
}
