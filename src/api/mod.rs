use actix_web::Scope;
use goodmorning_services::api::{accounts, jobs, storage, triggers, usercontent};

mod blue;
mod generic;

pub fn scope() -> Scope {
    Scope::new("api")
        .service(generic::scope())
        .service(blue::scope())
        .service(accounts::scope())
        .service(jobs::scope())
        .service(storage::scope())
        .service(triggers::scope())
        .service(usercontent::scope())
}
