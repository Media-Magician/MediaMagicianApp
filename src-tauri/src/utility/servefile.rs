use std::io::{Read, Seek, SeekFrom};
use tauri::{http::{header, Request, Response, StatusCode}, AppHandle, Manager};
use tauri_plugin_fs::{FsExt, OpenOptions};

use crate::structs::VideoRegistry;

const CHUNK: u64 = 5 * 1024 * 1024; // max 5MB chunks at once

fn err(status: StatusCode) -> Response<Vec<u8>> {
    Response::builder().status(status).body(Vec::new()).unwrap()
}

pub fn serve(app: &AppHandle, req: &Request<Vec<u8>>) -> Response<Vec<u8>> {
    let id = req.uri().path().trim_start_matches('/');
    let Some(src) = app.state::<VideoRegistry>().0.lock().unwrap().get(id).cloned() else { return err(StatusCode::NOT_FOUND) };

    let mut opts = OpenOptions::new();
    opts.read(true);
    let Ok(mut file) = app.fs().open(src, opts) else { return err(StatusCode::INTERNAL_SERVER_ERROR) };
    let Ok(len) = file.metadata().map(|m| m.len()) else { return err(StatusCode::INTERNAL_SERVER_ERROR) };

    // we treat no bytes header as bytes=0- and ranges without an end we limit to  the max chunk size defined at the top of the script
    let (start, end) = req.headers().get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("bytes="))
        .and_then(|v| v.split_once("-"))
        .map(|(s, e)| (s.parse::<u64>().unwrap_or(0), e.parse::<u64>().ok()))
        .unwrap_or((0, None));

    if start >= len { return err(StatusCode::RANGE_NOT_SATISFIABLE); }
    let end = end.unwrap_or(u64::MAX).min(start + CHUNK - 1).min(len - 1);
    if start > end { return err(StatusCode::RANGE_NOT_SATISFIABLE); }

    let mut buf = vec![0u8; (end - start + 1) as usize];
    if file.seek(SeekFrom::Start(start)).is_err() || file.read_exact(&mut buf).is_err() {
        return err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    Response::builder()
        .status(StatusCode::PARTIAL_CONTENT)
        .header(header::CONTENT_TYPE, "video/mp4") // we just respond that it's video/mp4 for simplicity, webview should manage fine anyway
        .header(header::ACCEPT_RANGES, "bytes")
        .header(header::CONTENT_RANGE, format!("bytes {start}-{end}/{len}"))
        .header(header::CONTENT_LENGTH, buf.len())
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .header(header::CACHE_CONTROL, "no-store")
        .header(header::ETAG, format!("\"{id}-{len}\""))
        .body(buf)
        .unwrap()
}
