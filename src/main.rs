mod file_utils;
mod file_cache;
mod error_400_utils;
mod css_utils;

use actix_cors::Cors;
use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{cookie::Key, middleware, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use std::io;
use std::path::Path;
use actix_web::web::route;

#[tokio::main]
async fn main() -> io::Result<()> {
    let css_dir = "./static";
    let output_file = "./static/all.css";

    // Primera combinación inicial
    // if let Err(e) = css_utils::combine_css(css_dir, output_file).await {
    //     eprintln!("Error inicial al combinar CSS: {}", e);
    // }
    //
    // // Iniciar el monitoreo de cambios
    // tokio::spawn(async move {
    //     if let Err(e) = css_utils::monitor_changes(css_dir, output_file).await {
    //         eprintln!("Error en el monitoreo de cambios: {}", e);
    //     }
    // });

    env_logger::init(); // Inicializa logs

    // Configuración de direcciones y puertos
    let http_addr = "127.0.0.1:80";
    let https_addr = "127.0.0.1:443";

    tokio::join!(
        start_http_server(http_addr),
        //start_https_server(https_addr)
    );

    Ok(())
}


async fn start_http_server(addr: &str) -> io::Result<()> {
    HttpServer::new(app_factory)
        .bind(addr)?
        .workers(8)
        .max_connections(50_000)
        .max_connection_rate(1_000)
        .client_request_timeout(std::time::Duration::from_secs(30))
        .client_disconnect_timeout(std::time::Duration::from_secs(5))
        .run()
        .await
}

// async fn start_https_server(addr: &str) -> io::Result<()> {
//     HttpServer::new(app_factory)
//         .bind_openssl(addr, load_ssl_keys()?)?
//         .workers(8)
//         .max_connections(50_000)
//         .max_connection_rate(1_000)
//         .client_request_timeout(std::time::Duration::from_secs(30))
//         .client_disconnect_timeout(std::time::Duration::from_secs(5))
//         .run()
//         .await
// }

fn app_factory() -> App() {
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
        .route("/static/{filename:.*}", web::get().to(static_files))
        .default_service(web::route().to(not_found))
        .app_data(web::JsonConfig::default().error_handler(|err, _req| {
            actix_web::error::InternalError::from_response(
                err,
                error_400_utils::handle_400_error().into(),
            )
            .into()
        }))
}

// Función para cargar certificados SSL
// fn load_ssl_keys() -> std::io::Result<openssl::ssl::SslAcceptor> {
//     use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};
//
//     let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
//     builder.set_private_key_file("certs/private.key", SslFiletype::PEM)?;
//     builder.set_certificate_chain_file("certs/certificate.crt")?;
//     Ok(builder.build())
// }


async fn static_files(req: HttpRequest) -> HttpResponse {
    let filename: String = req.match_info().query("filename").parse().unwrap();
    let path = format!("./static/{}", filename);

    println!("RUTA GENÉRICA: Solicitado: {}, Mapeado a: {}", filename, path);

    if Path::new(&path).exists() {
        file_cache::file_handler(&path)
    } else {
        HttpResponse::NotFound().body("Archivo no encontrado")
    }
}

async fn index_page(session: Session) -> impl Responder {
    if session.get::<String>("auth_token").unwrap_or(None).is_some() {
        file_cache::file_handler("./static/index/index.html")
    } else {
        HttpResponse::Found()
            //.append_header(("Location", "/login"))
            .append_header(("login", "./login/login.html"))
            .finish()
    }
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



async fn index_script() -> HttpResponse {
    file_cache::file_handler("./static/index/index_script.js")
}

// async fn login_page() -> HttpResponse {
//     //file_cache::file_handler(".static/login/login.html")
//     HttpResponse::Found()
//         .append_header(("login", ".static/login/login.html"))
//         .finish()
// }

async fn login_page() -> HttpResponse {
    let path = "./static/login/login.html"; // Ruta completa al archivo

    // Verificar si el archivo existe
    if !Path::new(path).is_file() {
        eprintln!("Archivo no encontrado: {}", path);
        return HttpResponse::NotFound().body("Archivo no encontrado");
    }

    println!("Sirviendo archivo desde /login: {}", path);
    file_cache::file_handler(path) // Sirve el archivo
}


async fn login_script() -> HttpResponse {
    file_cache::file_handler("./static/login/login_script.js")
}

async fn allcss_page() -> HttpResponse {
    file_cache::file_handler("./all.css")
}

// Página de error 404
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


// Generar clave secreta para las sesiones
fn secret_key() -> Key {
    Key::from(&[0; 64])
}

