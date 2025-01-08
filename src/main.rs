use actix_files as fs;
use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use serde::Serialize;
use tokio_postgres::{NoTls};

// Estructura para representar datos
#[derive(Serialize)]
struct Item {
    id: i32,
    name: String,
    description: String,
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

    // Crear tabla si no existe
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS items (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT NOT NULL
            )",
            &[],
        )
        .await?;

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

// Ruta para servir los datos como JSON
async fn items() -> impl Responder {
    let client = connect_db().await.unwrap();
    let items = get_items(&client).await.unwrap();
    HttpResponse::Ok().json(items) // Devuelve los datos como JSON
}

// Ruta principal
async fn index() -> impl Responder {
    HttpResponse::Found()
        .append_header(("Location", "/static/index.html"))
        .finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index)) // Ruta principal
            .route("/items", web::get().to(items)) // Ruta para obtener datos como JSON
            .service(fs::Files::new("/static", "./static").index_file("index.html")) // Archivos estáticos
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}



/*use actix_web::{web, App, HttpServer, HttpResponse, Responder};

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello, Actix-Web!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index)) // Define una ruta para probar
    })
        .bind("127.0.0.1:80")?
        .run()
        .await
}
*/

/*
use actix_files as fs; // Para servir archivos estáticos
use actix_web::{web, App, HttpServer, HttpResponse, Responder};

async fn index() -> impl Responder {
    HttpResponse::Found()
        .append_header(("Location", "/static/index.html")) // Redirige al archivo HTML
        .finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index)) // Ruta principal
            .service(fs::Files::new("/static", "./static").show_files_listing()) // Servir archivos estáticos
    })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
*/

/*
use std::time::Instant;

fn main() {
    let start = Instant::now();
    let resultado = funcion_a_evaluar(2_000_000_000);
    let duration = start.elapsed();
    println!("Resultado: {}", resultado);
    println!("Tiempo transcurrido: {:.2?}", duration);
}

fn funcion_a_evaluar(n: u64) -> u64 {
    (0..n).sum()
}
*/