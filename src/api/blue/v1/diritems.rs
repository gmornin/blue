use std::{error::Error, path::PathBuf};

use actix_web::{get, web::Path, HttpResponse};
use bluemap_singleserve::Map;
use goodmorning_services::{
    bindings::services::v1::{V1Error, V1Response},
    functions::{dir_items, from_res, get_user_dir},
    structs::Account,
};

#[get("/diritems/{token}/{path:.*}")]
pub async fn diritems(path: Path<(String, String)>) -> HttpResponse {
    from_res(diritems_task(path).await)
}

async fn diritems_task(path: Path<(String, String)>) -> Result<V1Response, Box<dyn Error>> {
    let (token, path) = path.into_inner();
    let account = Account::v1_get_by_token(&token)
        .await?
        .v1_contains(&goodmorning_services::structs::GMServices::Blue)?
        .v1_restrict_verified()?;

    let mut items = Vec::new();

    let base = PathBuf::from(&path);
    let base_abs = get_user_dir(account.id, None).join(path);

    for parent in base.ancestors() {
        if Map::exists(&base_abs.join(parent)).await {
            return Err(V1Error::TypeMismatch.into());
        }
    }

    for item in dir_items(account.id, &base, true, false).await? {
        if !item.is_file && Map::exists(&base_abs.join(&item.name)).await {
            items.push(item);
        }
    }

    Ok(V1Response::DirContent { content: items })
}
