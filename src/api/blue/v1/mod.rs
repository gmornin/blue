use actix_web::Scope;

mod diritems;
mod presets;
mod render;

pub fn scope() -> Scope {
    Scope::new("v1")
        .service(diritems::diritems)
        .service(render::render)
        .service(presets::presets)
}
