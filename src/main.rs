mod file_utils;
mod file_cache;
mod error_400_utils;

use actix_cors::Cors;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{cookie::Key, middleware, web, App, HttpResponse, HttpServer, Responder};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::io;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::sync::{mpsc};


#[tokio::main]
async fn main() -> io::Result<()> {
    let css_dir = "./static";
    let output_file = "./static/all.css";

    // Primera combinación inicial
    if let Err(e) = combine_css(css_dir, output_file).await {
        eprintln!("Error inicial al combinar CSS: {}", e);
    }

    // Iniciar el monitoreo de cambios
    tokio::spawn(async move {
        if let Err(e) = monitor_changes(css_dir, output_file).await {
            eprintln!("Error en el monitoreo de cambios: {}", e);
        }
    });

    // Iniciar el servidor web
    HttpServer::new(|| {
        App::new()
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key(),
            ))
            .wrap(
                Cors::default()
                    .allowed_origin("https://example.com")
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![actix_web::http::header::CONTENT_TYPE])
                    .max_age(3600),
            )
            .wrap(middleware::DefaultHeaders::new()
                .add(("X-Custom-Header", "Value"))
                .add(("Strict-Transport-Security", "max-age=63072000; includeSubDomains"))
                .add(("X-Frame-Options", "DENY"))
                .add(("X-Content-Type-Options", "nosniff")))
            .wrap(middleware::Compress::default())
            .route("/", web::get().to(index_page))
            .route("/index.js", web::get().to(index_script))
            .route("/login", web::get().to(login_page))
            .route("/login.js", web::get().to(login_script))
            .route("/all.css", web::get().to(allcss_page))
            .route("/antigua-url", web::get().to(redirect_301))
            .route("/temporal-url", web::get().to(redirect_302))
            .route("/items", web::get().to(items_handler))
            .default_service(web::route().to(not_found)) // Manejo 404
            .app_data(web::JsonConfig::default().error_handler(|err, _req| {
                actix_web::error::InternalError::from_response(
                    err,
                    error_400_utils::handle_400_error().into() )
                    .into()
            }))
    })
        .bind("127.0.0.1:80")?
        .workers(8)
        .max_connections(50_000)
        .max_connection_rate(1_000)
        .client_request_timeout(std::time::Duration::from_secs(30))
        .client_disconnect_timeout(std::time::Duration::from_secs(5))
        .run()
        .await
}

// Página principal protegida
// async fn index_page(session: Session) -> impl Responder {
//     if session.get::<String>("auth_token").unwrap_or(None).is_some() {
//         HttpResponse::Ok()
//             .append_header(("Cache-Control", "max-age=31536000")) // 1 año
//             .append_header(("ETag", "custom-etag-value"))
//             .body("Página principal protegida")
//
//     } else {
//         HttpResponse::Found()
//             .append_header(("Location", "/login"))
//             .finish()
//     }
// }

async fn index_page(session: Session) -> impl Responder {
    if session.get::<String>("auth_token").unwrap_or(None).is_some() {
        file_cache::file_handler("./static/index/index.html")
    } else {
        HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish()
    }
}

async fn index_script() -> HttpResponse {
    file_cache::file_handler("./static/index/index_script.js")
}

async fn login_page() -> HttpResponse {
    file_cache::file_handler("./static/login/login.html")
}

async fn login_script() -> HttpResponse {
    file_cache::file_handler("./static/login/login_script.js")
}

async fn allcss_page() -> HttpResponse {
    file_cache::file_handler("./static/all.css")
} // Página de error 404

async fn not_found() -> impl Responder {
    HttpResponse::NotFound().body("404 Página no encontrada")
}

async fn redirect_301() -> HttpResponse {
    HttpResponse::MovedPermanently()
        .append_header(("Location", "/nuevo-destino"))
        .finish()
}

async fn redirect_302() -> HttpResponse {
    HttpResponse::Found()
        .append_header(("Location", "/temporal-destino"))
        .finish()
}
// fn file_handler() -> HttpResponse {
//     HttpResponse::Ok()
//         .append_header(("Cache-Control", "max-age=31536000")) // 1 año
//         .append_header(("ETag", "custom-etag-value"))
//         .body("Contenido del archivo estático")
// }

async fn items_handler(session: Session) -> impl Responder {
    if session.get::<String>("auth_token").unwrap_or(None).is_some() {
        HttpResponse::Ok().json(vec![
            "Item 1",
            "Item 2",
            "Item 3",
        ]) // Devuelve una lista de ítems como JSON
    } else {
        HttpResponse::Unauthorized().body("No autorizado")
    }
}

// Monitorear cambios en la carpeta de CSS
async fn monitor_changes(css_dir: &str, output_file: &str) -> io::Result<()> {
    println!("Monitoreando cambios en '{}'", css_dir);

    let (tx, mut rx) = mpsc::channel(1);

    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                let _ = tx.try_send(event);
            }
        },
        Default::default(),
    )
        .unwrap();

    watcher
        .watch(Path::new(css_dir), RecursiveMode::Recursive)
        .unwrap();

    while let Some(event) = rx.recv().await {
        match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                println!("Detectado cambio: {:?}", event);
                if let Err(e) = combine_css(css_dir, output_file).await {
                    eprintln!("Error al combinar CSS: {}", e);
                }
            }
            _ => {
                println!("Evento ignorado: {:?}", event.kind);
            }
        }
    }

    Ok(())
}

// Combinar todos los archivos CSS en uno
async fn combine_css(css_dir: &str, output_file: &str) -> io::Result<()> {
    let mut output = tokio::fs::File::create(output_file).await?;
    let mut stack = vec![Path::new(css_dir).to_path_buf()];

    while let Some(current_dir) = stack.pop() {
        let mut entries = tokio::fs::read_dir(&current_dir).await?;
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("css") {
                let content = tokio::fs::read_to_string(&path).await?;
                output
                    .write_all(format!("/* {} */\n", path.display()).as_bytes())
                    .await?;
                output.write_all(content.as_bytes()).await?;
                println!("CSS añadido: {}", path.display());
            }
        }
    }

    println!("CSS combinado en '{}'", output_file);
    Ok(())
}


// Generar clave secreta para las sesiones
fn secret_key() -> Key {
    Key::from(&[0; 64])
}

