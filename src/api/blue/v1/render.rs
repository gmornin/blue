use std::{error::Error, ffi::OsStr, path::PathBuf, time::Duration};

use actix_web::{
    post,
    web::{self, Json},
    HttpResponse,
};
use goodmorning_services::{
    bindings::services::v1::{V1Error, V1Render, V1Response},
    functions::{from_res, get_user_dir, has_dotdot},
    structs::{Account, GMServices, Jobs},
    MAX_CONCURRENT,
};
use tokio::fs;

use crate::structs::RenderTask;

#[post("/render")]
pub async fn render(post: Json<V1Render>, jobs: web::Data<Jobs>) -> HttpResponse {
    from_res(render_task(post, jobs).await)
}

async fn render_task(
    post: Json<V1Render>,
    jobs: web::Data<Jobs>,
) -> Result<V1Response, Box<dyn Error>> {
    let post = post.into_inner();

    let mut account = Account::v1_get_by_token(&post.token)
        .await?
        .v1_restrict_verified()?
        .v1_contains(&goodmorning_services::structs::GMServices::Blue)?;

    let symlinked_account = false;

    let mut from_path = PathBuf::from(post.from.trim_start_matches('/'));
    let mut to_path = std::path::Path::new("blue").join(post.to.trim_start_matches('/'));
    let to_path_original = to_path.clone();

    if let [_, "Shared", user, ..] = from_path
        .iter()
        .map(|s| s.to_str().unwrap())
        .collect::<Vec<_>>()
        .as_slice()
    {
        account = if let Some(account) = Account::find_by_username(user.to_string()).await? {
            account
                .v1_restrict_verified()?
                .v1_contains(&goodmorning_services::structs::GMServices::Blue)?
        } else {
            return Err(V1Error::FileNotFound.into());
        };
        from_path = [from_path.iter().next().unwrap()]
            .into_iter()
            .chain(from_path.iter().skip(3))
            .collect();
    }

    if let ["Shared", user, ..] = to_path
        .iter()
        .map(|s| s.to_str().unwrap())
        .collect::<Vec<_>>()
        .as_slice()
    {
        if !user.eq_ignore_ascii_case(&account.username) {
            return Err(V1Error::PermissionDenied.into());
        } else {
            to_path = [to_path.iter().next().unwrap()]
                .into_iter()
                .chain(to_path.iter().skip(2))
                .collect();
        }
    }

    if has_dotdot(&from_path)
        || has_dotdot(&to_path)
        || from_path
            .iter()
            .nth(1)
            .is_some_and(|p| p == OsStr::new(".system"))
        || to_path
            .iter()
            .next()
            .is_some_and(|p| p == OsStr::new(".system"))
        || has_dotdot(&PathBuf::from(&post.preset))
    {
        return Err(V1Error::PermissionDenied.into());
    }

    if fs::try_exists(&get_user_dir(account.id, Some(GMServices::Blue)).join(&to_path)).await? {
        return Err(V1Error::PathOccupied.into());
    }

    let mut res = jobs
        .run_with_limit(
            account.id,
            Box::new(RenderTask {
                from: from_path,
                to: to_path,
                user: account.id,
                preset: post.preset.trim_start_matches('/').to_string(),
            }),
            *MAX_CONCURRENT.get().unwrap(),
            goodmorning_services::bindings::structs::ApiVer::V1,
            Duration::from_secs(900),
        )
        .await
        .as_v1()?;

    if symlinked_account {
        if let V1Response::BlueRendered { newpath, .. } = &mut res {
            *newpath = to_path_original.to_string_lossy().to_string()
        }
    }

    Ok(res)
}
