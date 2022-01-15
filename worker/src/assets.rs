use std::borrow::Cow;
use wasm_bindgen::prelude::*;
use worker::{kv::KvStore, Request, Response};

//#[link(wasm_import_module = "./custom.js")]
#[wasm_bindgen(raw_module = "./assets.mjs")]
extern "C" {
    fn get_asset(name: &str) -> Option<String>;
}

pub fn resolve(name: &str) -> Cow<'_, str> {
    match get_asset(name) {
        Some(name) => Cow::Owned(name),
        None => Cow::Borrowed(name),
    }
}

pub fn is_asset_path(path: &str) -> bool {
    let last_segment = path.rsplit_once("/").map(|x| x.1).unwrap_or(path);
    last_segment.contains('.')
}

pub async fn serve_asset(req: Request, store: KvStore) -> worker::Result<Response> {
    let path = req.path();
    let path = path.trim_start_matches('/');
    let path = resolve(path);
    let value = match store.get(&path).bytes().await? {
        Some(value) => value,
        None => return Response::error("Not Found", 404),
    };
    #[allow(clippy::redundant_clone)]
    let mut response = Response::from_bytes(value.clone())?;
    response
        .headers_mut()
        .set("Content-Type", get_mime(&path).unwrap_or("text/plain"))?;
    Ok(response)
}

fn get_mime(path: &str) -> Option<&'static str> {
    let ext = if let Some((_, ext)) = path.rsplit_once(".") {
        ext
    } else {
        return None;
    };

    let ct = match ext {
        "html" => "text/html",
        "css" => "text/css",
        "js" => "text/javascript",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "ico" => "image/x-icon",
        "wasm" => "application/wasm",
        _ => return None,
    };

    Some(ct)
}
