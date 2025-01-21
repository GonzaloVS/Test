use std::{fs, io};
use actix_web::{web, App, HttpResponse, HttpServer};

#[tokio::main]
async fn main() -> io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .route("/", web::get().to(index_page))
    })
        .bind("127.0.0.1:80")?
        .run()
        .await
}

async fn index_page() ->HttpResponse {
    let path ="./static/index/index.html";
    match fs::read(path) {
        Ok(file_content) => {
            let mime_type = mime_guess::from_path(path).first_or_octet_stream();
            HttpResponse::Ok()
                .content_type(mime_type.as_ref())
                .body(file_content)
        }
        Err(_) => HttpResponse::NotFound().body(custom_404()),
    }

}

async fn custom_404() -> HttpResponse {
    match fs::read("./static/404.html") {
        Ok(content) => HttpResponse::NotFound()
            .content_type("text/html")
            .body(content),
        Err(_) => HttpResponse::NotFound().body("404 - Página no encontrada"),
    }
}