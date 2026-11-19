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

use std::path::PathBuf;

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

fn androidPickFile(
    app: tauri::AppHandle,
    reg: State<'_, VideoRegistry>,
    id: String,
) -> Result<PickedFile, String> {
    use crate::structs::Progress;
    use std::io::Write;
    use tauri::Emitter;
    use tauri_plugin_fs::FsExt;

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

    let id = uuid::Uuid::new_v4().to_string();
    reg.0.lock().unwrap().insert(id.clone(), FilePath::Path(targetPath.clone()));

    Ok(PickedFile {
        path: targetPath.into(),
        id: Some(id),
        name: originalFile.name,
    })
}

#[tauri::command]
pub async fn pick_file(
    app: tauri::AppHandle,
    reg: State<'_, VideoRegistry>,
    id: String,
    _path: Option<String>,
) -> Result<Video, String> {
    println!("Picking file");

    let pickedFile = androidPickFile(app, reg, id.clone())?;
    let videoStats = getVideoStats(pickedFile.path.to_string())?;

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
    code: String,
    name: String,
    chunk_b64: String,
    eof: bool,
    state: State<'_, FileReaders>,
) -> Result<String, String> {
    let mut filePath = PathBuf::from("/storage/emulated/0/Download");
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

        scan_media_file(&pathStr).map_err(|e| e.to_string())?;
    }

    Ok(pathStr)
}

fn scan_media_file(path: &str) -> Result<(), String> {
    use jni::objects::{JObject, JValue};

    // get app context
    let ctx = ndk_context::android_context();
    let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }.map_err(|e| e.to_string())?;

    // attach to the current jvm thread
    let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
    let context = unsafe { JObject::from_raw(ctx.context().cast()) };

    let jPath = env.new_string(path).map_err(|e| e.to_string())?;
    let emptyStr = env.new_string("").map_err(|e| e.to_string())?;

    // create String[] array from file path
    let stringClass = env.find_class("java/lang/String").map_err(|e| e.to_string())?;
    let stringArray = env.new_object_array(1, stringClass, emptyStr).map_err(|e| e.to_string())?;
    env.set_object_array_element(&stringArray, 0, jPath).map_err(|e| e.to_string())?;

    // calling mediascanner with the context and the path
    let mediaScannerClass = env.find_class("android/media/MediaScannerConnection").map_err(|e| e.to_string())?;

    env.call_static_method(
        mediaScannerClass,
        "scanFile",
        "(Landroid/content/Context;[Ljava/lang/String;[Ljava/lang/String;Landroid/media/MediaScannerConnection$OnScanCompletedListener;)V",
        &[
            JValue::Object(&context),
            JValue::Object(&stringArray),
            JValue::Object(&JObject::null()), // null mimetype (android should auto-detect)
            JValue::Object(&JObject::null()), // null callback
        ],
    ).map_err(|e| e.to_string())?;

    Ok(())
}
