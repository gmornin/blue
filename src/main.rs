use actix_web::{App, HttpServer};

#[actix_web::main]
async fn main() {
    HttpServer::new(|| App::new());
}
