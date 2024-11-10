use actix_files::NamedFile;
use actix_web::{get, web::Path, Result};
use goodmorning_services::SERVICES_STATIC;

use crate::values::BLUE_CONFIG;

#[get("/static/{path:.*}")]
pub async fn r#static(params: Path<String>) -> Result<NamedFile> {
    let params = params.into_inner();

    Ok(NamedFile::open_async(
        std::path::Path::new(&BLUE_CONFIG.get().unwrap().static_path)
            .join(params.trim_start_matches('/')),
    )
    .await?)
}

#[get("/static/services/{path:.*}")]
pub async fn static_services(params: Path<String>) -> Result<NamedFile> {
    let params = params.into_inner();

    Ok(NamedFile::open_async(
        SERVICES_STATIC
            .get()
            .unwrap()
            .join(params.trim_start_matches('/')),
    )
    .await?)
}
