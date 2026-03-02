#![allow(non_snake_case)]

use tauri::Manager;
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
    // TODO: Add code here that during setup clears old files from the cache dir

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default();
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
            eprintln!("cleaned cache dir");
            
            // Register all deep links
            // #[cfg(any(windows, target_os = "linux"))]
            // app.deep_link().register_all()?;
            #[cfg(any(windows, target_os = "linux"))]
            if let Err(e) = app.deep_link().register_all() {
                eprintln!("Warning: failed to register deep links: {}", e);
            }
            eprintln!("registered deep links");
            
            // Check whether there is a 'current' deep link
            let start_urls = app.deep_link().get_current()?;
            if let Some(urls) = start_urls {
                eprintln!("deep link URLs: {:?}", urls);
            }
            eprintln!("got start urls");
            
            // Log deep link triggers
            app.deep_link().on_open_url(|event| {
                let urls = event.urls();
                for url in urls {
                    eprintln!("URL: {}", url);
                }
            });
            eprintln!("registered deep link callback");

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
