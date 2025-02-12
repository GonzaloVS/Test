use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::Serialize;
use sqlx::PgPool;

// Estructura para los elementos de la tabla "items"
#[derive(Serialize)]
struct Item {
    id: i32,
    name: String,
    description: Option<String>,
}

// Manejador para obtener todos los ítems desde PostgreSQL
async fn get_items(db_pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as::<_, Item>("SELECT id, name, description FROM items")
        .fetch_all(db_pool.get_ref()) // Obtener conexión desde el pool
        .await;

    match result {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(err) => {
            eprintln!("Error al consultar la base de datos: {:?}", err);
            HttpResponse::InternalServerError().body("Error al obtener los datos")
        }
    }
}

async fn api_handler(path: web::Path<String>) -> impl Responder {
    let tail = path.into_inner();
    HttpResponse::Ok().json(serde_json::json!({
        "message": format!("Este es un archivo JSON para la ruta /api/{}", tail)
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // Configurar la conexión a la base de datos PostgreSQL
    let database_url = "postgres://gonzo:ZNTn3H2^)^88@57.128.171.242:5432/trancosdb";
    // Crear el pool de conexiones
    let db_pool = PgPool::connect(database_url)
        .await
        .expect("Error al conectar a PostgreSQL");

    HttpServer::new(|| {
        App::new()
            //.route("/api/{tail:.*}", web::get().to(api_handler))
            .app_data(web::Data::new(db_pool.clone())) // Pasar pool de conexiones a la app
            .route("/api/items", web::get().to(get_items)) // Ruta para obtener los items
            .route("/api/{_:.*}", web::get().to(api_handler))
    })
        //.bind("127.0.0.1:8000")?
        .bind("0.0.0.0:8000")? // Escucha en todas las interfaces IPv4 e IPv6 (por caddy)
        .run()
        .await
}
