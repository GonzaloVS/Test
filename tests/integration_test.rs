// tests/integration_test.rs
use actix_web::{test, web, App, HttpResponse, Responder};

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Hello, Actix-Web!")
}

#[actix_web::test]
async fn test_index_connection() {
    let app = test::init_service(
        App::new().route("/", web::get().to(index))
    ).await;

    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    let body = test::read_body(resp).await;
    assert_eq!(body, "Hello, Actix-Web!");
}
