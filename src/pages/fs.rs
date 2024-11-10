use std::{borrow::Cow, error::Error, path::PathBuf};

use actix_files::NamedFile;
use actix_web::{get, http::header::ContentType, web::Path, HttpRequest, HttpResponse};
use bluemap_singleserve::Map;
use goodmorning_services::bindings::services::v1::V1Error;
use goodmorning_services::{
    functions::{dir_items, get_user_dir},
    structs::{Account, GMServices},
    traits::CollectionItem,
    ACCOUNTS,
};
use tokio::fs;

use crate::{
    components::{self, topbar_from_req, FsItem, FsItemProp, PathProp},
    functions::{from_res, gen_nonce},
    values::{BLUE_CONFIG, CSP_BASE},
};

#[get("/fs/{id}/{path:.*}")]
pub async fn fspath(path: Path<(i64, String)>, req: HttpRequest) -> HttpResponse {
    from_res(fs_task(path, &req).await, &req).await
}

#[get("/fs/{id}")]
pub async fn root(path: Path<i64>, req: HttpRequest) -> HttpResponse {
    from_res(
        fs_task(Path::from((path.into_inner(), String::new())), &req).await,
        &req,
    )
    .await
}

async fn fs_task(
    path: Path<(i64, String)>,
    req: &HttpRequest,
) -> Result<HttpResponse, Box<dyn Error>> {
    let (id, path) = path.into_inner();

    let (topbar, account) = match topbar_from_req(req).await? {
        Ok(stuff) => stuff,
        Err(res) => return Ok(res),
    };

    let is_owner = account.as_ref().is_some_and(|account| account.id == id);

    let mut account = if let Some(account) = account
        && account.id == id
    {
        account
    } else {
        match Account::find_by_id(id, ACCOUNTS.get().unwrap()).await? {
            Some(account) => account,
            None => {
                return Ok(NamedFile::open_async(
                    std::path::Path::new(&BLUE_CONFIG.get().unwrap().static_path)
                        .join("html/notfound.html"),
                )
                .await?
                .into_response(req))
            }
        }
    };

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

    // get_user_dir(account.id, None).join(&path);
    let pathbuf = get_user_dir(account.id, Some(GMServices::Tex)).join(&preview_path);

    if matches!(path.as_str(), "Shared" | "Shared/") {
        return dir(account, id, path, topbar, is_owner).await;
    }

    if !fs::try_exists(&pathbuf).await? {
        return Err(V1Error::FileNotFound.into());
    }

    dir(account, id, path, topbar, is_owner).await
}

async fn dir(
    account: Account,
    id: i64,
    path: String,
    topbar: Cow<'_, str>,
    is_owner: bool,
) -> Result<HttpResponse, Box<dyn Error>> {
    let pathbuf = std::path::Path::new("blue").join(&path);
    let mut items = Vec::new();

    let base_abs = get_user_dir(account.id, None).join(&pathbuf);

    for parent in pathbuf.ancestors() {
        if Map::exists(&base_abs.join(parent)).await {
            return Ok(HttpResponse::TemporaryRedirect()
                .append_header(("Location", format!("/fs/{id}/{path}")))
                .await
                .unwrap());
        }
    }

    for mut item in dir_items(account.id, &pathbuf, true, false).await? {
        if Map::exists(&base_abs.join(&item.name)).await {
            item.is_file = true;
            items.push(item);
        } else if !item.is_file {
            items.push(item);
        }
    }

    let nonce = gen_nonce();
    let csp_header = format!("{} 'nonce-{nonce}'", CSP_BASE.get().unwrap());
    let items_props = FsItemProp {
        nonce,
        id,
        items: items.into_iter().map(FsItem::from).collect(),
        path: path.clone(),
    };
    let items_display = yew::ServerRenderer::<components::FsItems>::with_props(|| items_props)
        .render()
        .await;
    let path_props = PathProp {
        path: path.trim_end_matches('/').to_string(),
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
        )
        || !is_owner
    {
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
  <!-- <script src="/static/scripts/upload.js" defer></script> -->
  </body>
</html>"#,
        html_escape::encode_safe(&format!("{}/{path}", id))
    );

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .insert_header(("Content-Security-Policy", csp_header))
        .body(html))
}
