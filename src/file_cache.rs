use actix_web::{HttpResponse};
use std::collections::HashMap;
use std::sync::RwLock;
use sha2::{Digest};
use lazy_static::lazy_static;
use crate::file_utils;

lazy_static! {
    static ref FILE_CACHE: RwLock<HashMap<String, (String, Vec<u8>)>> = RwLock::new(HashMap::new());
}

pub(crate) fn file_handler(file_path: &str) -> HttpResponse {
    let cache = FILE_CACHE.read().unwrap();
    if let Some((etag, content)) = cache.get(file_path) {
        // Si ya está en caché, devolvemos la respuesta
        return HttpResponse::Ok()
            .append_header(("Cache-Control", "max-age=31536000")) // 1 año
            .append_header(("ETag", etag.as_str()))
            .body(content.clone());
    }
    drop(cache); // Liberamos el lock antes de escribir

    // Si no está en caché, lo cargamos
    match file_utils::load_file(file_path) {
        Ok((etag, content)) => {
            let mut cache = FILE_CACHE.write().unwrap();
            cache.insert(file_path.to_string(), (etag.clone(), content.clone()));
            HttpResponse::Ok()
                .append_header(("Cache-Control", "max-age=31536000")) // 1 año
                .append_header(("ETag", etag.as_str()))
                .append_header(("ETagError","hola"))
                .body(content)
        }
        Err(_) => HttpResponse::InternalServerError().body("Error al cargar el archivo"),
    }
}
