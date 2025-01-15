use actix_web::{HttpResponse};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::RwLock;
use actix_web::http::header::ContentType;
use lazy_static::lazy_static;
use mime::Mime;
use crate::file_utils;

lazy_static! {
    static ref FILE_CACHE: RwLock<HashMap<String, (String, Vec<u8>)>> = RwLock::new(HashMap::new());
}

// pub(crate) fn file_handler(file_path: &str) -> HttpResponse {
//     let cache = FILE_CACHE.read().unwrap();
//     if let Some((etag, content)) = cache.get(file_path) {
//         // Si ya está en caché, devolvemos la respuesta
//         println!("{:?}, {:?}, {:?}", cache, etag, content.len());
//         return build_response(&etag, &content, file_path);
//     }
//     drop(cache); // Liberamos el lock antes de escribir
//
//     // Si no está en caché, lo cargamos
//     match file_utils::load_file(file_path) {
//         Ok((etag, content)) => {
//             let mut cache = FILE_CACHE.write().unwrap();
//             println!("{:?}", cache);
//             cache.insert(file_path.parse().unwrap(), (etag.clone(), content.clone()));
//             build_response(&etag, &content, file_path)
//         }
//         Err(_) => HttpResponse::InternalServerError().body("Error al cargar el archivo"),
//     }
// }

pub(crate) fn file_handler(file_path: &str) -> HttpResponse {
    use std::fs;
    use std::path::Path;

    // Normalizar la ruta para evitar claves inconsistentes en el caché
    let normalized_path = match Path::new(file_path).canonicalize() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error al normalizar la ruta '{}': {}", file_path, e);
            return HttpResponse::InternalServerError().body(format!("Error al acceder al archivo: {}", e));
        }
    };

    let normalized_path_str = match normalized_path.to_str() {
        Some(path_str) => path_str.to_string(),
        None => {
            eprintln!("Error al convertir la ruta a cadena: {:?}", normalized_path);
            return HttpResponse::InternalServerError().body("Error al procesar la ruta del archivo");
        }
    };

    // Verificar el caché
    let cache = FILE_CACHE.read().unwrap();
    if let Some((etag, content)) = cache.get(&normalized_path_str) {
        println!("Archivo encontrado en caché: {} ({} bytes)", normalized_path_str, content.len());
        return build_response(etag, content, &normalized_path_str);
    }
    drop(cache); // Liberar el lock antes de escribir en el caché

    // Cargar desde el disco si no está en caché
    match file_utils::load_file(&normalized_path_str) {
        Ok((etag, content)) => {
            // Insertar en el caché
            let mut cache = FILE_CACHE.write().unwrap();
            cache.insert(normalized_path_str.clone(), (etag.clone(), content.clone()));
            println!("Archivo cargado y añadido al caché: {}", normalized_path_str);
            build_response(&etag, &content, &normalized_path_str)
        }
        Err(e) => {
            eprintln!("Error al cargar el archivo '{}': {}", normalized_path_str, e);
            HttpResponse::InternalServerError().body(format!("Error al cargar el archivo: {}", e))
        }
    }
}


fn build_response(etag: &str, content: &[u8], file_path: &str) -> HttpResponse {

    println!("{:?}, {:?}, {:?}",etag,  content, file_path);

    // Determinar el tipo de contenido basado en la extensión del archivo
    let content_type = match file_path.split('.').last() {
        Some("html") => ContentType::html(),
        Some("css") => ContentType(mime::TEXT_CSS),
        Some("js") => ContentType(mime::APPLICATION_JAVASCRIPT),
        Some("png") => ContentType(mime::IMAGE_PNG),
        Some("jpg") | Some("jpeg") => ContentType(mime::IMAGE_JPEG),
        Some("svg") => ContentType(Mime::from_str("image/svg+xml").unwrap()),
        _ => ContentType(mime::TEXT_PLAIN),
    };
    println!("{:?}, {:?}, {:?}",etag,  content, file_path);

    HttpResponse::Ok()
        .append_header(("Cache-Control", "max-age=31536000")) // 1 año
        .append_header(("ETag", etag))
        .content_type(content_type)
        .body(content.to_vec())
}