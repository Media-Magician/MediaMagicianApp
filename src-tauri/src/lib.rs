#![allow(non_snake_case)]

use std::{fs::{self, create_dir_all}, io::Write, path::Path};
use sha2::{Digest, Sha256};
#[allow(unused_imports)]
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_fs::{FsExt, FilePath::Url};

mod structs;
mod util;
use crate::structs::Notification;
#[allow(unused_imports)]
use crate::{structs::Progress, util::processVideo};

#[tauri::command]
fn sha256_digest(app: tauri::AppHandle, chunk: Vec<u8>, id: String) -> Result<Vec<u8>, String> {
    let _ = app.emit("progress", Progress { r#type: "completed".into(), stage: "hashing".into(), progress: None, id: id.clone() });
    
    let mut hasher = Sha256::new();
    hasher.update(&chunk);
    
    Ok(hasher.finalize().to_vec())
}

// #[cfg(target_os = "android")]
async fn triggerAndroidFilePicker(app: tauri::AppHandle, id: String) -> Result<Option<String>, String> {
    let fPath = app.dialog().file().blocking_pick_file();
    if fPath.is_none() {
        return Err("No file selected".into());
    }

    let resolver = app.path();
    let mut filePath = fPath.unwrap();
    // TODO: Add handling for passing file name to frontend
    let _fileName = match filePath {
        Url(ref url) => resolver.file_name(url.as_str()).unwrap(),
        _ => resolver.file_name(filePath.as_path().unwrap().to_str().unwrap()).unwrap(),
    };

    let _ = app.emit("progress", Progress { r#type: "completed".into(), stage: "copying".into(), progress: None, id: id.clone() });

    let file = app.fs().read(filePath.clone());
    if file.is_err() { 
        return Err("Failed to read file".into());
    }

    filePath = app.path().app_cache_dir().unwrap().join(filePath.to_string().split("/").last().unwrap()).into();
    println!("File exists?: {}", filePath.as_path().unwrap().exists());
        
    let targetFile = std::fs::OpenOptions::new().write(true).create(true).open(filePath.clone().into_path().unwrap());
    println!("File exists?: {}", filePath.as_path().unwrap().exists());
    if targetFile.is_err() {
        return Err("Failed to create target file".into());
    }
        
    let mut targetFile = targetFile.unwrap();
    targetFile.write_all(&file.unwrap()).unwrap();
    targetFile.flush().unwrap();
    println!("File exists?: {}", filePath.as_path().unwrap().exists());
    println!("Selected file: {:#?}", filePath);
    
    let mut outputName = filePath.as_path().unwrap().file_name().unwrap().to_str().unwrap().to_owned();
    if !outputName.contains(".") {
        outputName.push_str(".mp4");
    }

    let outputPath = app.path().app_cache_dir().unwrap().as_mut_os_str().to_str().unwrap().to_owned() + "/" + outputName.as_str();
    println!("Output file path: {}; exists: {}; parent exists: {}", outputPath, Path::new(&outputPath).exists(), Path::new(&outputPath).parent().unwrap().exists());

    create_dir_all(Path::new(&outputPath).parent().unwrap()).unwrap();

    let _ = app.emit("progress", Progress { r#type: "progress".into(), stage: "processing".into(), progress: Some(0.0), id: id.clone() });
    let err = processVideo(
        filePath.clone().into_path().unwrap().as_mut_os_str().to_str().unwrap().into(),
        outputPath.clone(),
        |p| {
            let _ = app.emit("progress", Progress { r#type: "progress".into(), stage: "processing".into(), progress: Some(p), id: id.clone() });
        }
    ).is_err();
    let _ = app.emit("notify", if err {
        Notification { title: "Error".into(), message: "Failed to process video file.".into() }
    } else {
        Notification { title: "Success".into(), message: "Video file processed successfully. Preparing to upload.".into() }
    });

    let _ = fs::remove_file(filePath.into_path().unwrap()).is_ok();
    
    if err { 
        return Err("Failed to process video file".into());
    }
    let _ = app.emit("progress", Progress { r#type: "progress".into(), stage: "processing".into(), progress: Some(1.0), id: id.clone() });

    return Ok(outputPath.into());
}

// come back to this one, probably should do it with ffmpeg sidecar
#[cfg(not(target_os = "android"))]
async fn triggerNonAndroidFilePicker(app: tauri::AppHandle, id: String) -> Result<Option<String>, String> {
    let fPath = app.dialog().file().blocking_pick_file();
    if fPath.is_none() {
        return Err("No file selected".into());
    }

    let resolver = app.path();
    let mut filePath = fPath.unwrap();
    let fileName = match filePath {
        Url(ref url) => resolver.file_name(url.as_str()).unwrap(),
        _ => resolver.file_name(filePath.as_path().unwrap().to_str().unwrap()).unwrap(),
    };

    println!("Selected file: {:#?}", filePath);

    let mut outputName = filePath.as_path().unwrap().file_name().unwrap().to_str().unwrap().to_owned();
    if !outputName.contains(".") {
        outputName.push_str(".mp4");
    }

    let outputPath = app.path().app_cache_dir().unwrap().as_mut_os_str().to_str().unwrap().to_owned() + "/" + outputName.as_str();
    println!("Output file path: {}; exists: {}; parent exists: {}", outputPath, Path::new(&outputPath).exists(), Path::new(&outputPath).parent().unwrap().exists());

    create_dir_all(Path::new(&outputPath).parent().unwrap()).unwrap();

    app.emit("progress", Progress { r#type: "progress".into(), stage: "processing".into(), progress: Some(0.0), id: id.clone() });
    processVideo(
        filePath.into_path().unwrap().as_mut_os_str().to_str().unwrap().into(),
        outputPath.clone(),
        |p| {
            app.emit("progress", Progress { r#type: "progress".into(), stage: "processing".into(), progress: Some(p), id: id.clone() });
        }
    ).expect("Failed to process video file");

    app.emit("progress", Progress { r#type: "progress".into(), stage: "processing".into(), progress: Some(1.0), id: id.clone() });

    return Ok(outputPath.into());
}

#[tauri::command]
async fn trigger_filepicker(app: tauri::AppHandle, id: String) -> Result<Option<String>, String> {
    #[cfg(target_os = "android")]
    return triggerAndroidFilePicker(app, id).await;

    #[cfg(not(target_os = "android"))]
    return triggerNonAndroidFilePicker(app, id).await;

    // let fPath = app.dialog().file().blocking_pick_file();
    // if fPath.is_none() {
    //     return Err("No file selected".into());
    // }

    // let resolver = app.path();
    // let mut filePath = fPath.unwrap();
    // let fileName = match filePath {
    //     Url(ref url) => resolver.file_name(url.as_str()).unwrap(),
    //     _ => resolver.file_name(filePath.as_path().unwrap().to_str().unwrap()).unwrap(),
    // };

    // #[cfg(target_os = "android")]
    // {
    //     app.emit("progress", Progress { r#type: "completed".into(), stage: "copying".into(), progress: None, id: id.clone() });

    //     let file = app.fs().read(filePath.clone());
    //     if file.is_err() { 
    //         return Err("Failed to read file".into());
    //     }

    //     filePath = app.path().app_cache_dir().unwrap().join(filePath.to_string().split("/").last().unwrap()).into();
    //     println!("File exists?: {}", filePath.as_path().unwrap().exists());
        
    //     let targetFile = std::fs::OpenOptions::new().write(true).create(true).open(filePath.clone().into_path().unwrap());
    //     println!("File exists?: {}", filePath.as_path().unwrap().exists());
    //     if targetFile.is_err() {
    //         return Err("Failed to create target file".into());
    //     }
        
    //     let mut targetFile = targetFile.unwrap();
    //     targetFile.write_all(&file.unwrap()).unwrap();
    //     targetFile.flush().unwrap();
    //     println!("File exists?: {}", filePath.as_path().unwrap().exists());
    // }

    // println!("Selected file: {:#?}", filePath);

    
    // let mut outputName = filePath.as_path().unwrap().file_name().unwrap().to_str().unwrap().to_owned();
    // if !outputName.contains(".") {
    //     outputName.push_str(".mp4");
    // }

    // let outputPath = app.path().app_cache_dir().unwrap().as_mut_os_str().to_str().unwrap().to_owned() + "/" + outputName.as_str();
    // println!("Output file path: {}; exists: {}; parent exists: {}", outputPath, Path::new(&outputPath).exists(), Path::new(&outputPath).parent().unwrap().exists());

    // create_dir_all(Path::new(&outputPath).parent().unwrap()).unwrap();

    // app.emit("progress", Progress { r#type: "progress".into(), stage: "processing".into(), progress: Some(0.0), id: id.clone() });
    // processVideo(
    //     filePath.into_path().unwrap().as_mut_os_str().to_str().unwrap().into(),
    //     outputPath.clone(),
    //     |p| {
    //         app.emit("progress", Progress { r#type: "progress".into(), stage: "processing".into(), progress: Some(p), id: id.clone() });
    //     }
    // ).expect("Failed to process video file");

    // app.emit("progress", Progress { r#type: "progress".into(), stage: "processing".into(), progress: Some(1.0), id: id.clone() });

    // return Ok(outputPath.into());
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("This is a wonky ass test.");
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![trigger_filepicker, sha256_digest])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
