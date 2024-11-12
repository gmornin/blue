use actix_web::{web::Data, App, HttpServer};
use gm_blue::{pages, r#static::r#static, values::BLUE_CONFIG};
use goodmorning_services::structs::Jobs;

#[actix_web::main]
async fn main() {
    goodmorning_services::init().await;
    gm_blue::values::init();

    let jobs: Data<Jobs> = Data::new(Jobs::default());

    HttpServer::new(move || {
        App::new()
            .service(r#static)
            .service(gm_blue::api::scope())
            .service(pages::home)
            .service(pages::render)
            .service(pages::fspath)
            .service(pages::root)
            .app_data(jobs.clone())
    })
    .bind(("0.0.0.0", BLUE_CONFIG.get().unwrap().port))
    .unwrap()
    .run()
    .await
    .unwrap();
}
