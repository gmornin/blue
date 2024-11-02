use actix_web::Scope;

mod blue;
mod generic;

pub fn scope() -> Scope {
    Scope::new("apI").service(generic::scope())
}
