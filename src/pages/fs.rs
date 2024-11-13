use std::{borrow::Cow, error::Error, path::PathBuf};

use actix_files::NamedFile;
use actix_web::http::header::HeaderValue;
use actix_web::routes;
use actix_web::{get, http::header::ContentType, web::Path, HttpRequest, HttpResponse};
use bluemap_singleserve::Map;
use goodmorning_services::bindings::services::v1::V1Error;
use goodmorning_services::functions::{cookie_to_str, get_usersys_dir};
use goodmorning_services::traits::CollectionItem;
use goodmorning_services::ACCOUNTS;
use goodmorning_services::{
    functions::{dir_items, get_user_dir},
    structs::{Account, GMServices},
};
use tokio::fs;

use crate::{
    components::{self, topbar_from_req, FsItem, FsItemProp, PathProp},
    functions::{from_res, gen_nonce},
    values::BLUE_CONFIG,
};

#[get("/fs/{path:.*}")]
pub async fn fspath(path: Path<String>, req: HttpRequest) -> HttpResponse {
    from_res(fs_task(path, &req).await, &req).await
}

#[routes]
#[get("/fs")]
#[get("/fs/")]
pub async fn root(req: HttpRequest) -> HttpResponse {
    from_res(fs_task(Path::from(String::new()), &req).await, &req).await
}

async fn fs_task(path: Path<String>, req: &HttpRequest) -> Result<HttpResponse, Box<dyn Error>> {
    let path = path.into_inner();

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
        account.v1_restrict_verified()?
    } else {
        return Ok(NamedFile::open_async(
            std::path::Path::new(&BLUE_CONFIG.get().unwrap().static_path).join("html/login.html"),
        )
        .await?
        .into_response(req));
    };

    let id = account.id;

    let mut preview_path = PathBuf::from(&path);

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

    if !account
        .services
        .contains(&GMServices::Blue.as_str().to_string())
    {
        let path = get_usersys_dir(account.id, Some(GMServices::Blue));
        fs::create_dir_all(&path).await?;

        account.services.push(GMServices::Blue.as_str().to_string());
        account.save_replace(ACCOUNTS.get().unwrap()).await?;
    }

    // get_user_dir(account.id, None).join(&path);
    let pathbuf = get_user_dir(account.id, Some(GMServices::Blue)).join(&preview_path);

    if !req
        .headers()
        .get("accept")
        .unwrap_or(&HeaderValue::from_str("html").unwrap())
        .to_str()
        .unwrap()
        .contains("html")
    {
        let pathbuf = std::path::Path::new("blue").join(&path);
        let base_abs = get_user_dir(account.id, None);

        if let &["blue", "Shared", username, ..] = pathbuf
            .iter()
            .map(|s| s.to_str().unwrap())
            .collect::<Vec<_>>()
            .as_slice()
        {
            if username.eq_ignore_ascii_case(&account.username) {
                let pathbuf = std::path::Path::new("blue")
                    .iter()
                    .chain(pathbuf.iter().skip(3))
                    .collect::<PathBuf>();
                for parent in pathbuf.ancestors() {
                    if Map::exists(&base_abs.join(parent)).await {
                        return Map::serve(
                            &base_abs.join(parent),
                            &pathbuf
                                .iter()
                                .skip(parent.iter().count())
                                .collect::<PathBuf>(),
                            req,
                        )
                        .await;
                    }
                }
            }
        } else {
            for parent in pathbuf.ancestors() {
                if Map::exists(&base_abs.join(parent)).await {
                    return Map::serve(
                        &base_abs.join(parent),
                        &pathbuf
                            .iter()
                            .skip(parent.iter().count())
                            .collect::<PathBuf>(),
                        req,
                    )
                    .await;
                }
            }
        }
    }

    if pathbuf.iter().last().map(|s| s.to_str().unwrap()) == Some("map")
        && Map::exists(
            &pathbuf
                .iter()
                .take(pathbuf.iter().count() - 1)
                .collect::<PathBuf>(),
        )
        .await
    {
        return Map::serve(&pathbuf, std::path::Path::new(""), req).await;
    }

    if Map::exists(&pathbuf).await {
        return map(account, path, topbar).await;
    }

    if matches!(path.as_str(), "Shared" | "Shared/") {
        return dir(account, id, path.clone(), path.to_string(), topbar).await;
    }

    if !fs::try_exists(&pathbuf).await? {
        return Err(V1Error::FileNotFound.into());
    }
    dir(
        account,
        id,
        preview_path.to_string_lossy().trim_matches('/').to_string(),
        path.to_string(),
        topbar,
    )
    .await
}

async fn map(
    account: Account,
    path: String,
    topbar: Cow<'_, str>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let path_escaped = html_escape::encode_safe(&path).to_string();
    let map_path_dirty = format!("/fs/{}/map", path.trim_matches('/'));
    let map_path = html_escape::encode_text(&map_path_dirty);

    let path_display = yew::ServerRenderer::<components::Path>::with_props(move || PathProp {
        id: account.id,
        path: path.trim_end_matches('/').to_string(),
    })
    .render()
    .await;

    let id = account.id;
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
    <link rel="stylesheet" href="/static/css/file-previews.css" />
    <link rel="stylesheet" href="/static/css/topbar-loggedin.css" />
    <link rel="stylesheet" href="/static/css/dark/main.css" />
    <link rel="stylesheet" href="/static/css/dark/topbar.css" />
    <link rel="stylesheet" href="/static/css/dark/path.css" />
    <link rel="stylesheet" href="/static/css/dark/preview.css" />
    <link
      rel="shortcut icon"
      href="/static/images/logo.webp"
      type="image/x-icon"
    />
    <title>{id}/{path_escaped}</title>
  </head>
  <body>
    {topbar}
<div id="path-display">
    {path_display}
</div>
    <iframe id="viewer" src="{map_path}"></iframe> 
    <script src="/static/scripts/file.js" defer></script>
    <script src="/static/scripts/topbar.js" defer></script>
  </body>
</html>"#,
    );

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        // .insert_header(("Content-Security-Policy", CSP_BASE.get().unwrap().as_str()))
        .body(html))
}

async fn dir(
    account: Account,
    id: i64,
    path: String,
    path_original: String,
    topbar: Cow<'_, str>,
) -> Result<HttpResponse, Box<dyn Error>> {
    let pathbuf = std::path::Path::new("blue").join(&path);
    let mut items = Vec::new();

    let base_abs = get_user_dir(account.id, None).join(&pathbuf);

    // for parent in pathbuf.ancestors().skip(1) {
    //     if Map::exists(&base_abs.join(parent)).await {
    //         return Ok(HttpResponse::TemporaryRedirect()
    //             .append_header(("Location", format!("/fs/{path}")))
    //             .await
    //             .unwrap());
    //     }
    // }

    for mut item in dir_items(account.id, &pathbuf, true, false).await? {
        if id != account.id
            && matches!(
                std::path::Path::new(&path_original)
                    .join(&item.name)
                    .iter()
                    .map(|s| s.to_str().unwrap())
                    .collect::<Vec<_>>()
                    .as_slice(),
                &["Shared", _, "Shared", ..]
            )
        {
            continue;
        }

        if Map::exists(&base_abs.join(&item.name)).await {
            item.is_file = true;
            items.push(item);
        } else if !item.is_file {
            items.push(item);
        }
    }

    let nonce = gen_nonce();
    let items_props = FsItemProp {
        prepend: if id != account.id {
            Some(format!("{id}/Shared/{}", account.username))
        } else {
            None
        },
        nonce,
        id: account.id,
        items: items.into_iter().map(FsItem::from).collect(),
        path: path.clone(),
    };
    let items_display = yew::ServerRenderer::<components::FsItems>::with_props(|| items_props)
        .render()
        .await;
    let path_props = PathProp {
        path: path_original.clone(),
        id,
    };
    let path_display = yew::ServerRenderer::<components::Path>::with_props(|| path_props)
        .render()
        .await;
    let upload = if path.starts_with(".system")
        || matches!(path.as_str(), "Shared" | "Shared/")
        || matches!(
            path.split('/').collect::<Vec<_>>().as_slice(),
            ["Shared", _, ".system", ..]
        ) {
        r#"<img src="/static/icons/fileadd.svg" alt="" width="20px" height="20px" id="create" style="display: none;" /><img src="/static/icons/upload.svg" width="20px" height="20px" id="upload" style="display: none;" />"#
    } else {
        r#"<img src="/static/icons/fileadd.svg" width="20px" height="20px" id="create" /><img src="/static/icons/upload.svg" width="20px" height="20px" id="upload" />"#
    };
    let pathbuf_safe = html_escape::encode_safe(pathbuf.to_str().unwrap());

    let html = format!(
        r#"<!-- {{ "path": "{pathbuf_safe}", "id": {id} }} -->
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <link rel="stylesheet" href="/static/css/main.css" />
    <link rel="stylesheet" href="/static/css/topbar.css" />
    <link rel="stylesheet" href="/static/css/topbar-signedout.css" />
    <link rel="stylesheet" href="/static/css/fs.css" />
    <link rel="stylesheet" href="/static/css/path.css" />
    <link rel="stylesheet" href="/static/css/topbar-loggedin.css" />
    <link rel="stylesheet" href="/static/css/dark/main.css" />
    <link rel="stylesheet" href="/static/css/dark/topbar.css" />
    <link rel="stylesheet" href="/static/css/dark/fs.css" />
    <link rel="stylesheet" href="/static/css/dark/path.css" />
    <link rel="stylesheet" href="/static/css/dark/topbar-signedout.css" />
    <link
      rel="shortcut icon"
      href="/static/images/logo.webp"
      type="image/x-icon"
    />
    <title>{}</title>
  </head>
  <body>
    <dialog id="copyd">
      <div class="x">&#x2715;</div>
      <h2>Copy item</h2>
      <center><img src="/static/icons/copy.svg" height="50px" /></center>
      <p id="copy-from">Copy from: <span></span></p>
      <input type="text" id="copytarget" placeholder="Copy target" />
      <button id="copybut" class="submitbut">Copy</button>
    </dialog>
    <dialog id="moved">
      <div class="x">&#x2715;</div>
      <h2>Move item</h2>
      <center><img src="/static/icons/folder-tree.svg" height="50px" /></center>
      <p id="move-from">Move from: <span></span></p>
      <input type="text" id="movetarget" placeholder="Move to" />
      <button id="movebut" class="submitbut">Move</button>
    </dialog>
    <dialog id="restored">
      <div class="x">&#x2715;</div>
      <h2>Head up!</h2>
      <center><img src="/static/icons/warn.svg" height="50px" /></center>
      <center><p>There is a file at target location</p></center>
      <button id="restorebut" class="dangerbut">Overwrite file</button>
    </dialog>
  {topbar}
<div id="path-display">
  {path_display}
  <!-- <div id="pathitems">{upload}</div> -->
</div>
  {items_display}
  <script src="/static/scripts/fs.js" defer></script>
  <script src="/static/scripts/topbar.js" defer></script>
  <!-- <script src="/static/scripts/upload.js" defer></script> -->
  </body>
</html>"#,
        html_escape::encode_safe(&format!("{}/{path_original}", id))
    );

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        // .insert_header(("Content-Security-Policy", csp_header))
        .body(html))
}
