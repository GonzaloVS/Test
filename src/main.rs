use actix_files::NamedFile;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{cookie::Key, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use lazy_static::lazy_static;
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::collections::HashSet;
use std::io;
use std::path::Path;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use uuid::Uuid;

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
            // Habilitar compresión automática (Gzip, Deflate, Brotli)
            .wrap(middleware::Compress::default())
            // Middleware de sesiones
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key(),
            ))
            .route("/", web::get().to(index_page))
            .route("/index.js", web::get().to(index_script))
            .route("/login", web::get().to(login_page))
            .route("/login.js", web::get().to(login_script))
            .route("/all.css", web::get().to(allcss_page))
            .default_service(web::route().to(not_found)) // Manejo 404
    })
        .bind("127.0.0.1:80")?
        .workers(8)              // Ajustar segun los nucleos del servidor
        .max_connections(50_000) // 50,000 conexiones activas
        .max_connection_rate(1_000) // Hasta 1,000 conexiones nuevas por segundo
        .run()
        .await
}


// Página principal protegida
async fn index_page(session: Session) -> impl Responder {
    if session.get::<String>("auth_token").unwrap_or(None).is_some() {
        HttpResponse::Ok().body("Página principal protegida")
    } else {
        HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish()
    }
}

async fn index_script() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./static/index/index_script.js")?)
}

async fn login_page() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./static/login/login.html")?)
}

async fn login_script() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./static/login/login_script.js")?)
}

async fn allcss_page() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./static/all.css")?)
}

// Página de error 404
async fn not_found() -> impl Responder {
    HttpResponse::NotFound().body("404 Página no encontrada")
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
