use std::{error::Error, path::PathBuf};

use actix_web::{get, web::Path, HttpResponse};
use bluemap_singleserve::Map;
use goodmorning_services::{
    bindings::services::v1::{V1Error, V1Response},
    functions::{dir_items, from_res, get_user_dir},
    structs::{Account, GMServices},
};

#[get("/diritems/{token}/{path:.*}")]
pub async fn diritems(path: Path<(String, String)>) -> HttpResponse {
    from_res(diritems_task(path).await)
}

async fn diritems_task(path: Path<(String, String)>) -> Result<V1Response, Box<dyn Error>> {
    let (token, path) = path.into_inner();

    let mut account = Account::v1_get_by_token(&token)
        .await?
        .v1_contains(&goodmorning_services::structs::GMServices::Blue)?
        .v1_restrict_verified()?;

    let mut preview_path = PathBuf::from(&path);
    let id = account.id;

    if let ["Shared", user, ..] = preview_path
        .iter()
        .map(|s| s.to_str().unwrap())
        .collect::<Vec<_>>()
        .as_slice()
    {
        account = if let Some(account) = Account::find_by_username(user.to_string()).await? {
            account.v1_restrict_verified()?
        } else {
            return Err(V1Error::FileNotFound.into());
        };
        preview_path = preview_path.iter().skip(2).collect();
    }

    let mut items = Vec::new();

    let base = std::path::Path::new("blue").join(&path);
    let base_abs = get_user_dir(account.id, Some(GMServices::Blue)).join(&preview_path);

    for parent in base.ancestors() {
        if Map::exists(&base_abs.join(parent)).await {
            return Err(V1Error::TypeMismatch.into());
        }
    }

    for mut item in dir_items(id, &base, true, false).await? {
        if Map::exists(&base_abs.join(&item.name)).await {
            item.is_file = true;
            items.push(item);
        } else if !item.is_file {
            items.push(item);
        }
    }

    Ok(V1Response::DirContent { content: items })
}
