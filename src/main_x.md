

use actix_web::{get, post, web, App, HttpServer, Responder};

#[get("/")]
async fn index() -> impl Responder {
    "Hello, Rust Chat App!\n"
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    req_body
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(echo)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}