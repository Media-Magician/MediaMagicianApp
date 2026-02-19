#![allow(non_snake_case)]

use std::{fs::create_dir_all, io::Write, path::Path, time::Instant};
use sha2::{Digest, Sha256};
#[allow(unused_imports)]
use tauri::{Emitter, Manager};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_fs::FsExt;

mod structs;
mod util;
#[allow(unused_imports)]
use crate::{structs::Progress, util::processVideo};

#[tauri::command]
fn sha256_digest(chunk: Vec<u8>) -> Result<Vec<u8>, String> {
    let start = Instant::now();
    let mut hasher = Sha256::new();
    hasher.update(&chunk);
    println!("Digest took: {} ms", start.elapsed().as_millis());
    Ok(hasher.finalize().to_vec())
}

#[tauri::command]
async fn trigger_filepicker(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let fPath = app.dialog().file().blocking_pick_file();
    if fPath.is_none() {
        return Err("No file selected".into());
    }

    let resolver = app.path();
    let mut filePath = fPath.unwrap();
    let fileName = resolver.file_name(&filePath.as_path().as_ref().unwrap().to_str().unwrap());

    #[cfg(target_os = "android")]
    {
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
    }

    println!("Selected file: {:#?}", filePath);

    
    let mut outputName = filePath.as_path().unwrap().file_name().unwrap().to_str().unwrap().to_owned();
    if !outputName.contains(".") {
        outputName.push_str(".mp4");
    }

    let outputPath = app.path().app_cache_dir().unwrap().as_mut_os_str().to_str().unwrap().to_owned() + "/" + outputName.as_str();
    println!("Output file path: {}; exists: {}; parent exists: {}", outputPath, Path::new(&outputPath).exists(), Path::new(&outputPath).parent().unwrap().exists());

    create_dir_all(Path::new(&outputPath).parent().unwrap()).unwrap();
    processVideo(filePath.into_path().unwrap().as_mut_os_str().to_str().unwrap().into(), outputPath.clone()).expect("Failed to process video file");


    return Ok(outputPath.into());
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
