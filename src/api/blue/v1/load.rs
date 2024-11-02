use std::error::Error;

use actix_web::{get, web::Path, HttpResponse};
use goodmorning_services::{bindings::services::v1::V1Response, functions::from_res};

#[get("/load/{token}/{userid}/{userpath}")]
pub async fn load(path: Path<(String, i64, String)>) -> HttpResponse {
    let (token, userid, userpath) = path.into_inner();
    from_res(load_task(token, userid, userpath).await)
}

pub async fn load_task(
    token: String,
    userid: i64,
    userpath: String,
) -> Result<V1Response, Box<dyn Error>> {
    todo!()
}
