use std::fmt::Write;
use std::{error::Error, path::PathBuf};

use actix_files::NamedFile;
use actix_web::{get, http::header::ContentType, web, HttpRequest, HttpResponse};
use bluemap_singleserve::Map;
use goodmorning_services::structs::Account;
use goodmorning_services::{
    bindings::services::v1::V1Error,
    functions::{cookie_to_str, get_user_dir},
    structs::GMServices,
};
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::{
    components::topbar_from_req,
    functions::from_res,
    values::{BLUE_CONFIG, PRESETS},
};

#[derive(Serialize, Deserialize)]
struct Query {
    target: String,
    source: String,
}

#[get("/render")]
pub async fn render(req: HttpRequest, query: web::Query<Query>) -> HttpResponse {
    from_res(render_task(&req, query).await, &req).await
}

async fn render_task(
    req: &HttpRequest,
    query: web::Query<Query>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let token_cookie = req.cookie("token");
    let token = cookie_to_str(&token_cookie);

    if token.is_none() {
        return Ok(NamedFile::open_async(
            std::path::Path::new(&BLUE_CONFIG.get().unwrap().static_path).join("html/login.html"),
        )
        .await
        .map(|file| file.into_response(req))?);
    }

    let (topbar, account) = match topbar_from_req(req).await? {
        Ok(stuff) => stuff,
        Err(res) => return Ok(res),
    };

    let mut account = if let Some(account) = account {
        account
    } else {
        return Ok(NamedFile::open_async(
            std::path::Path::new(&BLUE_CONFIG.get().unwrap().static_path).join("html/login.html"),
        )
        .await?
        .into_response(req));
    };

    let mut source_path = PathBuf::from(&query.source.trim_matches('/'));
    let mut target_path = PathBuf::from(&query.target.trim_matches('/'));

    if let [service, "Shared", user, ..] = source_path
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
        target_path = target_path.iter().skip(2).collect();
        source_path = std::path::Path::new(service)
            .iter()
            .chain(source_path.iter().skip(3))
            .collect();
    }

    if Map::exists(&get_user_dir(account.id, Some(GMServices::Blue)).join(&target_path)).await {
        return Ok(HttpResponse::TemporaryRedirect()
            .insert_header((
                "Location",
                format!("/fs/{}", query.target.trim_start_matches('/')),
            ))
            .await
            .unwrap());
    }

    if !fs::try_exists(get_user_dir(account.id, None).join(&source_path)).await? {
        return Err(V1Error::FileNotFound.into());
    }

    if fs::try_exists(get_user_dir(account.id, Some(GMServices::Blue)).join(&target_path)).await? {
        return Err(V1Error::PathOccupied.into());
    }

    let source_safe = html_escape::encode_safe(query.source.trim_matches('/'));
    let target_safe = html_escape::encode_safe(query.target.trim_matches('/'));

    let selected = &BLUE_CONFIG.get().unwrap().default_preset;
    let all_presets = PRESETS
        .get()
        .unwrap()
        .iter()
        .fold(String::new(), |mut buf, current| {
            write!(buf, r#"<option value="{current}">{current}</option>"#).unwrap();
            buf
        });

    let html = format!(
        r#"
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <link rel="stylesheet" href="/static/css/main.css" />
    <link rel="stylesheet" href="/static/css/topbar.css" />
    <link rel="stylesheet" href="/static/css/path.css" />
    <link rel="stylesheet" href="/static/css/preview.css" />
    <link rel="stylesheet" href="/static/css/render.css" />
    <link rel="stylesheet" href="/static/css/topbar-loggedin.css" />
    <link rel="stylesheet" href="/static/css/dark/main.css" />
    <link rel="stylesheet" href="/static/css/dark/topbar.css" />
    <link rel="stylesheet" href="/static/css/dark/path.css" />
    <link rel="stylesheet" href="/static/css/dark/preview.css" />
    <link rel="stylesheet" href="/static/css/dark/render.css" />
    <link
      rel="shortcut icon"
      href="/static/images/logo.webp"
      type="image/x-icon"
    />
    <title>Render to BlueMap - GM Blue</title>
  </head>
  <body>
    {topbar}
    <div id="tiles">
      <div id="left">
        <h1>Select a file</h1>
        <span
          >Source: <span class="path">{source_safe}</span></span
        >
        <span
          >Target:
          <span class="path">blue/{target_safe}</span></span
        >
      </div>
      <div id="mid">
        <h1>Choose a preset</h1>
        <select id="preset" selected="{selected}">
            {all_presets}
        </select>
      </div>
      <div id="right">
        <h1>Start rendering</h1>
        <button class="ghbutton" id="render">Render to BlueMap</button>
        <br />
        <br />
        <span id="timer" class="hide"></span>
        <span id="failed" class="hide"></span>
        <span id="success" class="hide"></span>
        <br />
        <button class="ghbutton dangerbut hide" id="viewerror">View error message</button>
        <button class="ghbutton hide" id="reload">Reload page</button>
      </div>
    </div>
    <script src="/static/scripts/render.js" defer></script>
  </body>
</html>"#,
    );

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        // .insert_header(("Content-Security-Policy", CSP_BASE.get().unwrap().as_str()))
        .body(html))
}
