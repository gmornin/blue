use actix_web::{get, HttpRequest, HttpResponse};
use goodmorning_services::{
    functions::cookie_to_str,
    structs::{Account, GMServices},
};

use crate::values::BLUE_CONFIG;

#[get("/")]
pub async fn home(req: HttpRequest) -> HttpResponse {
    let token_cookie = req.cookie("token");
    let token = cookie_to_str(&token_cookie);

    if token.is_none() {
        todo!("login page")
    }

    let account = match Account::find_by_token(token.unwrap()).await.unwrap() {
        Some(a) => a,
        None => return todo!("been logged out"),
    };

    if BLUE_CONFIG.get().unwrap().allow_create
        && !account
            .services
            .contains(&GMServices::Blue.as_str().to_string())
    {
        todo!("create")
    }

    todo!()
}
