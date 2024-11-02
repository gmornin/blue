use std::error::Error;

use actix_web::{post, web::Json, HttpResponse};
use goodmorning_services::{
    bindings::services::v1::{V1Response, V1SimpleUser, V1TokenOnly},
    functions::from_res,
    structs::Account,
    ACCOUNTS,
};
use mongodb::bson::doc;
use tokio_stream::StreamExt;

#[post("/users")]
pub async fn users(post: Json<V1TokenOnly>) -> HttpResponse {
    from_res(users_task(post).await)
}

pub async fn users_task(post: Json<V1TokenOnly>) -> Result<V1Response, Box<dyn Error>> {
    let account = Account::v1_get_by_token(&post.token).await?;

    let mut list = Vec::new();
    let mut cursor = ACCOUNTS
        .get()
        .unwrap()
        .find(doc! { "access.blue": account.id })
        .await?;

    while let Some(entry) = cursor.next().await {
        let entry = entry?;
        list.push(V1SimpleUser {
            username: entry.username,
            id: entry.id,
        });
    }

    Ok(V1Response::AllowedAccess { users: list })
}
