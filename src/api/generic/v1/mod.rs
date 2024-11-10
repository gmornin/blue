use actix_web::Scope;

mod create;
mod pfp;

pub fn scope() -> Scope {
    Scope::new("v1")
        .service(create::create)
        .service(pfp::pfp)
        .service(pfp::pfp_name)
}
