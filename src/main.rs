use actix_session::{storage::CookieSessionStore, Session, SessionMiddleware};
use actix_web::{cookie::Key, web, App, HttpResponse, HttpServer, Responder, http::header::ContentType};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tokio_postgres::NoTls;
use uuid::Uuid;

// Estructura para representar datos
#[derive(Serialize)]
struct Item {
    id: i32,
    name: String,
    description: String,
}

// Estructura para login
#[derive(Deserialize)]
struct LoginData {
    username: String,
    password: String,
}

// Función para conectar a la base de datos
async fn connect_db() -> Result<tokio_postgres::Client, tokio_postgres::Error> {
    let (client, connection) = tokio_postgres::connect(
        "host=57.128.171.242 port=5432 dbname=trancosdb user=gonzo password=ZNTn3H2^)^88",
        NoTls,
    ).await?;

    // Ejecuta la conexión en un thread separado
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    Ok(client)
}

// Función para obtener datos de la base de datos
async fn get_items(client: &tokio_postgres::Client) -> Result<Vec<Item>, tokio_postgres::Error> {
    let rows = client.query("SELECT id, name, description FROM items", &[]).await?;
    let items = rows
        .iter()
        .map(|row| Item {
            id: row.get(0),
            name: row.get(1),
            description: row.get(2),
        })
        .collect();
    Ok(items)
}


async fn items(session: Session) -> impl Responder {
    // Verificar si el token de autenticación existe en la sesión
    if let Some(_token) = session.get::<String>("auth_token").unwrap_or(None) {
        // Conectar a la base de datos y recuperar los datos
        let client = connect_db().await.unwrap();
        let items = get_items(&client).await.unwrap();
        // Devolver los datos como JSON
        HttpResponse::Ok().json(items)
    } else {
        // Si no hay token, denegar el acceso
        HttpResponse::Unauthorized().body("No autorizado")
    }
}



// Página de inicio de sesión
// async fn login_page() -> impl Responder {
//     let path: PathBuf = PathBuf::from("./static/login/login.html");
//     NamedFile::open(path).unwrap()
// }

// Página de inicio de sesión
async fn login_page() -> impl Responder {
    println!("login page");
    let html_path = PathBuf::from("./static/login/login.html");

    // Cargar contenido de los archivos
    let html_content = fs::read_to_string(html_path).unwrap_or_else(|_| "Error al cargar login.html".to_string());

    // Insertar el contenido CSS y JS en el HTML
    let response = html_content;

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(response)
}

// async fn login(session: Session, form: web::Form<LoginData>) -> impl Responder {
//     let username = form.username.clone();
//     let password = form.password.clone();
//
//     // Validación de credenciales (hardcodeado por simplicidad)
//     if username == "admin" && password == "1234" {
//         session.insert("user", username).unwrap();
//         HttpResponse::Ok().body("Login exitoso")
//     } else {
//         HttpResponse::Unauthorized().body("Credenciales incorrectas")
//     }
// }

// Validar login y crear sesión
async fn login(session: Session, form: web::Json<LoginData>) -> impl Responder {
    let username = form.username.clone();
    let password = form.password.clone();

    if username == "admin" && password == "1234" {
        // Generar un token único
        let token = Uuid::new_v4().to_string();

        // Guardar el token en la sesión del servidor
        session.insert("auth_token", token.clone()).unwrap();

        // Responder con el token
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(format!(r#"{{"token": "{}"}}"#, token))
    } else {
        HttpResponse::Unauthorized().body("Usuario o contraseña incorrectos")
    }
}
fn secret_key() -> Key {
    Key::generate()
}

// Ruta principal
async fn index_page(session: Session) -> impl Responder {
    // Verificar si el usuario está autenticado
    if session.get::<String>("auth_token").unwrap_or(None).is_some() {
        let html_path = PathBuf::from("./static/index/index.html");

        // Cargar contenido de los archivos
        let html_content = fs::read_to_string(html_path).unwrap_or_else(|_| "Error al cargar index.html".to_string());

        // Insertar el contenido CSS y JS en el HTML
        let response = html_content;

        HttpResponse::Ok()
            .content_type("text/html; charset=utf-8")
            .body(response)
    } else {
        // Redirigir a /login si el usuario no está autenticado
        HttpResponse::Found()
            .append_header(("Location", "/login"))
            .finish()
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Starting server at http://localhost:8080");
    HttpServer::new(|| {
        App::new()
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                secret_key(),
            ))
            .route("/", web::get().to(index_page)) // Ruta principal
            .route("/login", web::get().to(login_page)) // Página de login
            .route("/login", web::post().to(login)) // Procesar login
            .route("/items", web::get().to(items)) // Ruta para obtener datos como JSON
            })
        .bind("127.0.0.1:80")?
        .run()
        .await
}

