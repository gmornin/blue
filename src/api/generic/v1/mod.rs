use actix_web::Scope;

mod create;

pub fn scope() -> Scope {
    Scope::new("v1").service(create::create)
}
