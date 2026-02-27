#![allow(non_snake_case)]

use tauri_plugin_deep_link::DeepLinkExt;

mod structs;
mod commands {
    pub mod pickFile;
    pub mod wrapProcess;
    pub mod digest;
}
mod process {
    pub mod video;
}

use crate::commands::{digest::sha256_digest, pickFile::pick_file, wrapProcess::process_video};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // TODO: Add code here that during setup clears old files from the cache dir

    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            app.deep_link().on_open_url(|event| {
                let urls = event.urls();
                for url in urls {
                    println!("URL: {}", url);
                }
            });

            Ok(())
        })
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![process_video, sha256_digest, pick_file])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
