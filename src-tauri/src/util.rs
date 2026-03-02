use std::{
    fs::{self, read_dir},
    path::Path,
};

pub fn cleanCacheDir(cacheDir: impl AsRef<Path>) -> Result<(), String> {
    eprintln!("Cleaning cache dir: {:?}", cacheDir.as_ref());
    let cacheDir = cacheDir.as_ref();
    if !cacheDir.exists() {
        eprintln!("Cache dir does not exist");
        return Ok(());
    }
    eprintln!("Cache dir exists");
    
    for entry in read_dir(cacheDir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        
        if !path.is_dir() {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
    }
    eprintln!("Cache dir cleaned");

    Ok(())
}
