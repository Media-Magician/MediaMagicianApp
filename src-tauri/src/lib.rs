#![allow(non_snake_case)]

use tauri::{Cef, Manager};
// use tauri::Manager;
use tauri_plugin_deep_link::DeepLinkExt;

mod structs;
mod util;
mod commands {
    pub mod digest;
    pub mod files;
    pub mod wrapProcess;
}
mod process {
    pub mod video;
}

use crate::{
    commands::{
        digest::sha256_digest,
        files::{pick_file, remove_file},
        wrapProcess::process_video,
    },
    util::cleanCacheDir,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::<Cef>::default();
    // let mut builder = tauri::Builder::default();
    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = app.get_webview_window("main")
                .expect("no main window")
                .set_focus();
        }));
    }

    builder
        .plugin(tauri_plugin_deep_link::init())
        .setup(|app| {
            // Clean up files that may have stayed from last time
            let _ = cleanCacheDir(app.path().app_cache_dir().unwrap());
            
            // Register all deep links
            #[cfg(not(target_os = "android"))]
            if let Err(e) = app.deep_link().register_all() {
                println!("Warning: failed to register deep links: {}", e);
            }
            
            // Check whether there is a 'current' deep link
            let start_urls = app.deep_link().get_current()?;
            if let Some(urls) = start_urls {
                println!("deep link URLs: {:?}", urls);
            }
            
            // Log deep link triggers
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
        .invoke_handler(tauri::generate_handler![
            process_video,
            sha256_digest,
            pick_file,
            remove_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");

    eprintln!("After build");
}
