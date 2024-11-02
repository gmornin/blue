use std::error::Error;

use actix_web::{post, web::Json, HttpResponse};
use goodmorning_services::bindings::services::v1::{V1Error, V1Response, V1TokenOnly};
use goodmorning_services::{functions::*, structs::*, traits::CollectionItem, *};
use tokio::fs;

use crate::values::BLUE_CONFIG;

#[post("/create")]
pub async fn create(post: Json<V1TokenOnly>) -> HttpResponse {
    from_res(create_task(post).await)
}

async fn create_task(post: Json<V1TokenOnly>) -> Result<V1Response, Box<dyn Error>> {
    if !BLUE_CONFIG.get().unwrap().allow_create {
        return Err(V1Error::FeatureDisabled.into());
    }

    let mut account = Account::v1_get_by_token(&post.token)
        .await?
        .v1_restrict_verified()?
        .v1_not_contains(&GMServices::Tex)?;

    let path = get_usersys_dir(account.id, Some(GMServices::Blue));
    fs::create_dir_all(&path).await?;

    account.services.push(GMServices::Blue.as_str().to_string());
    account.save_replace(ACCOUNTS.get().unwrap()).await?;

    Ok(V1Response::ServiceCreated)
}
