use actix_files::NamedFile;
use actix_web::{get, HttpRequest, HttpResponse};
use goodmorning_services::{
    functions::cookie_to_str,
    structs::{Account, GMServices},
};

use crate::{functions::internalserver_error, intererr, values::BLUE_CONFIG};

#[get("/")]
pub async fn home(req: HttpRequest) -> HttpResponse {
    let token_cookie = req.cookie("token");
    let token = cookie_to_str(&token_cookie);

    if token.is_none() {
        return intererr!(NamedFile::open_async(
            std::path::Path::new(&BLUE_CONFIG.get().unwrap().static_path).join("html/login.html")
        )
        .await
        .map(|file| file.into_response(&req)));
    }

    let account = match Account::find_by_token(token.unwrap()).await {
        Ok(Some(account)) => account,
        Ok(None) => {
            return match NamedFile::open_async(
                std::path::Path::new(&BLUE_CONFIG.get().unwrap().static_path)
                    .join("html/been-loggedout.html"),
            )
            .await
            {
                Ok(file) => file.into_response(&req),
                Err(e) => internalserver_error(e.into()),
            }
        }
        Err(e) => {
            return internalserver_error(e.into());
        }
    };

    if !account
        .services
        .contains(&GMServices::Blue.as_str().to_string())
    {
        return match NamedFile::open_async(
            std::path::Path::new(&BLUE_CONFIG.get().unwrap().static_path)
                .join("html/finish-setup.html"),
        )
        .await
        {
            Ok(file) => file.into_response(&req),
            Err(e) => internalserver_error(e.into()),
        };
    }

    HttpResponse::TemporaryRedirect()
        .insert_header(("Location", "/fs"))
        .await
        .unwrap()
}
