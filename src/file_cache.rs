use actix_web::{HttpResponse};
use std::collections::HashMap;
use std::sync::RwLock;
use lazy_static::lazy_static;
use crate::file_utils;

lazy_static! {
    static ref FILE_CACHE: RwLock<HashMap<String, (String, Vec<u8>)>> = RwLock::new(HashMap::new());
}

pub(crate) fn file_handler(file_path: &str) -> HttpResponse {
    let cache = FILE_CACHE.read().unwrap();
    if let Some((etag, content)) = cache.get(file_path) {
        // Si ya está en caché, devolvemos la respuesta
        return build_response(&etag, &content);
    }
    drop(cache); // Liberamos el lock antes de escribir

    // Si no está en caché, lo cargamos
    match file_utils::load_file(file_path) {
        Ok((etag, content)) => {
            let mut cache = FILE_CACHE.write().unwrap();
            cache.insert(file_path.parse().unwrap(), (etag.clone(), content.clone()));
            build_response(&etag, &content)
        }
        Err(_) => HttpResponse::InternalServerError().body("Error al cargar el archivo"),
    }
}

fn build_response(etag: &str, content: &[u8]) -> HttpResponse {
    HttpResponse::Ok()
        .append_header(("Cache-Control", "max-age=31536000")) // 1 año
        .append_header(("ETag", etag))
        .body(content.to_vec()) }
