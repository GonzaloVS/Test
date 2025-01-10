use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpResponse, HttpServer, Responder};
use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::io;
use std::path::Path;
use tokio::sync::mpsc;
use tokio::io::AsyncWriteExt;
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
    println!("Starting server at http://127.0.0.1:8080");
    HttpServer::new(|| {
        App::new()
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key(),
            ))
            .route("/", web::get().to(index_page))
            .route("/login", web::get().to(login_page))
            .route("/login", web::post().to(login))
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}

// Función para monitorear cambios en la carpeta
async fn monitor_changes(css_dir: &str, output_file: &str) -> io::Result<()> {
    println!("Monitoreando cambios en '{}'", css_dir);

    // Crear un canal asincrónico para recibir eventos
    let (tx, mut rx) = mpsc::channel(1);

    // Configurar el watcher
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

    // Procesar eventos en un bucle asincrónico
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

// Función para combinar todos los archivos CSS
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

// Función para gestionar las rutas
async fn index_page(session: Session) -> impl Responder {
    if session.get::<String>("auth_token").unwrap_or(None).is_some() {
        HttpResponse::Ok().body("Página principal protegida")
    } else {
        HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish()
    }
}

async fn login_page() -> impl Responder {
    HttpResponse::Ok().body("Página de login")
}

async fn login(session: Session, form: web::Json<LoginData>) -> impl Responder {
    let username = form.username.clone();
    let password = form.password.clone();

    if username == "admin" && password == "1234" {
        let token = Uuid::new_v4().to_string();
        session.insert("auth_token", token.clone()).unwrap();
        HttpResponse::Ok().body("Login exitoso")
    } else {
        HttpResponse::Unauthorized().body("Credenciales inválidas")
    }
}

// Generar clave para sesiones
fn secret_key() -> actix_web::cookie::Key {
    Key::from(&[0; 32])
}

// Estructura para recibir datos de login
#[derive(serde::Deserialize)]
struct LoginData {
    username: String,
    password: String,
}


// use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
// use actix_web::{cookie::Key, web, App, HttpResponse, HttpServer, Responder, http::header::ContentType};
// use serde::{Deserialize, Serialize};
// use std::fs;
// use std::path::PathBuf;
// use tokio_postgres::NoTls;
// use uuid::Uuid;
//
//
// // Estructura para representar datos
// #[derive(Serialize)]
// struct Item {
//     id: i32,
//     name: String,
//     description: String,
// }
//
// // Estructura para login
// #[derive(Deserialize)]
// struct LoginData {
//     username: String,
//     password: String,
// }
//
//
//
// // Función para conectar a la base de datos
// async fn connect_db() -> Result<tokio_postgres::Client, tokio_postgres::Error> {
//     let (client, connection) = tokio_postgres::connect(
//         "host=57.128.171.242 port=5432 dbname=trancosdb user=gonzo password=ZNTn3H2^)^88",
//         NoTls,
//     ).await?;
//
//     // Ejecuta la conexión en un thread separado
//     tokio::spawn(async move {
//         if let Err(e) = connection.await {
//             eprintln!("Connection error: {}", e);
//         }
//     });
//
//     Ok(client)
// }
//
// // Función para obtener datos de la base de datos
// async fn get_items(client: &tokio_postgres::Client) -> Result<Vec<Item>, tokio_postgres::Error> {
//     let rows = client.query("SELECT id, name, description FROM items", &[]).await?;
//     let items = rows
//         .iter()
//         .map(|row| Item {
//             id: row.get(0),
//             name: row.get(1),
//             description: row.get(2),
//         })
//         .collect();
//     Ok(items)
// }
//
//
// async fn items(session: Session) -> impl Responder {
//     // Verificar si el token de autenticación existe en la sesión
//     if let Some(_token) = session.get::<String>("auth_token").unwrap_or(None) {
//         // Conectar a la base de datos y recuperar los datos
//         let client = connect_db().await.unwrap();
//         let items = get_items(&client).await.unwrap();
//         // Devolver los datos como JSON
//         HttpResponse::Ok().json(items)
//     } else {
//         // Si no hay token, denegar el acceso
//         HttpResponse::Unauthorized().body("No autorizado")
//     }
// }
//
//
//
// // Página de inicio de sesión
// // async fn login_page() -> impl Responder {
// //     let path: PathBuf = PathBuf::from("./static/login/login.html");
// //     NamedFile::open(path).unwrap()
// // }
//
// // Página de inicio de sesión
// // async fn login_page() -> impl Responder {
// //     let html_path = PathBuf::from("./static/login/login.html");
// //
// //     // Cargar contenido de los archivos
// //     let html_content = fs::read_to_string(html_path).unwrap_or_else(|_| "Error al cargar login.html".to_string());
// //
// //     // Insertar el contenido CSS y JS en el HTML
// //     let response = html_content;
// //
// //     HttpResponse::Ok()
// //         .content_type("text/html; charset=utf-8")
// //         .body(response)
// // }
//
// async fn login_page() -> impl Responder {
//     // Cargar el HTML, CSS y JS desde el sistema de archivos
//     let html_path = "./static/login/login.html";
//     let css_path = "./static/all.css";
//     let js_path = "./static/login/login_script.js";
//
//     let html_content = fs::read_to_string(html_path).unwrap_or_else(|_| "Error al cargar login.html".to_string());
//     let css_content = fs::read_to_string(css_path).unwrap_or_else(|_| "".to_string());
//     let js_content = fs::read_to_string(js_path).unwrap_or_else(|_| "".to_string());
//
//     // Inyectar el CSS y JS directamente en el HTML
//     let response = html_content
//         .replace("<!-- CSS_PLACEHOLDER -->", &format!("<style>{}</style>", css_content))
//         .replace("<!-- JS_PLACEHOLDER -->", &format!("<script>{}</script>", js_content));
//
//     HttpResponse::Ok()
//         .content_type("text/html; charset=utf-8")
//         .body(response)
// }
//
// // Validar login y crear sesión
// async fn login(session: Session, form: web::Json<LoginData>) -> impl Responder {
//     let username = form.username.clone();
//     let password = form.password.clone();
//
//     if username == "admin" && password == "1234" {
//         // Generar un token único
//         let token = Uuid::new_v4().to_string();
//
//         // Guardar el token en la sesión del servidor
//         session.insert("auth_token", token.clone()).unwrap();
//
//         // Responder con el token
//         HttpResponse::Ok()
//             .content_type(ContentType::json())
//             .body(format!(r#"{{"token": "{}"}}"#, token))
//     } else {
//         HttpResponse::Unauthorized().body("Usuario o contraseña incorrectos")
//     }
// }
// // fn secret_key() -> Key {
// //     Key::generate()
// // }
//
// fn secret_key() -> Key {
//     Key::from(&*b"mi-clave-secreta-segura-y-larga-32bytes.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a.a".to_vec())
// }
//
// // Ruta principal
// // async fn index_page(session: Session) -> impl Responder {
// //     // Verificar si el usuario está autenticado
// //     if session.get::<String>("auth_token").unwrap_or(None).is_some() {
// //         let html_path = PathBuf::from("./static/index/index.html");
// //
// //         // Cargar contenido de los archivos
// //         let html_content = fs::read_to_string(html_path).unwrap_or_else(|_| "Error al cargar index.html".to_string());
// //
// //         // Insertar el contenido CSS y JS en el HTML
// //         let response = html_content;
// //
// //         HttpResponse::Ok()
// //             .content_type("text/html; charset=utf-8")
// //             .body(response)
// //     } else {
// //         // Redirigir a /login si el usuario no está autenticado
// //         HttpResponse::Found()
// //             .append_header(("Location", "/login"))
// //             .finish()
// //     }
// // }
//
// async fn index_page(session: Session) -> impl Responder {
//     // Verificar si el usuario está autenticado
//     if session.get::<String>("auth_token").unwrap_or(None).is_some() {
//         let html_path = PathBuf::from("./static/index/index.html");
//         let css_path = "./static/all.css";
//         let js_path = "./static/index/index_script.js";
//         // Cargar contenido de los archivos
//         let html_content = fs::read_to_string(html_path).unwrap_or_else(|_| "Error al cargar index.html".to_string());
//         let css_content = fs::read_to_string(css_path).unwrap_or_else(|_| "".to_string());
//         let js_content = fs::read_to_string(js_path).unwrap_or_else(|_| "".to_string());
//
//         // Inyectar el CSS y JS directamente en el HTML
//         let response = html_content
//             .replace("<!-- CSS_PLACEHOLDER -->", &format!("<style>{}</style>", css_content))
//             .replace("<!-- JS_PLACEHOLDER -->", &format!("<script>{}</script>", js_content));
//
//
//         HttpResponse::Ok()
//             .content_type("text/html; charset=utf-8")
//             .body(response)
//     } else {
//         // Redirigir a /login si el usuario no está autenticado
//         HttpResponse::Found()
//             .append_header(("Location", "/login"))
//             .finish()
//     }
// }
//
//
// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     println!("Starting server at http://localhost:8080");
//     HttpServer::new(|| {
//         App::new()
//             .wrap(SessionMiddleware::new(
//                 CookieSessionStore::default(),
//                 secret_key(),
//             ))
//             .route("/", web::get().to(index_page)) // Ruta principal
//             .route("/login", web::get().to(login_page)) // Página de login
//             .route("/login", web::post().to(login)) // Procesar login
//             .route("/items", web::get().to(items)) // Ruta para obtener datos como JSON
//             })
//         .bind("127.0.0.1:80")?
//         .run()
//         .await
// }
//
