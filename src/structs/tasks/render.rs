use std::{fmt::Debug, path::PathBuf};

use async_trait::async_trait;
use bluemap_singleserve::{Config, Map};
use goodmorning_services::{
    bindings::{
        services::v1::{V1Error, V1Response},
        structs::*,
    },
    functions::get_user_dir,
    traits::TaskItem,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct RenderTask {
    pub from: PathBuf,
    pub to: PathBuf,
    pub preset: String,
    pub user: i64,
}

#[async_trait]
impl TaskItem for RenderTask {
    async fn run(&self, ver: &ApiVer, id: u64) -> CommonRes {
        let from_abs = get_user_dir(self.user, None).join(&self.from);
        let to_abs = get_user_dir(self.user, None).join(&self.to);

        match Map::render(
            &from_abs,
            &to_abs,
            &bluemap_singleserve::MasterConfig::get()
                .templates
                .join(&self.preset),
        )
        .await
        {
            Ok(()) => match ver {
                ApiVer::V1 => CommonRes::V1(Ok(V1Response::BlueRendered {
                    newpath: self.to.to_string_lossy().to_string(),
                    id,
                })),
            },
            Err(e) => match ver {
                ApiVer::V1 => CommonRes::V1(Err(V1Error::External {
                    content: e.to_string(),
                })),
            },
        }
    }

    fn to(&self, _ver: &ApiVer) -> Box<dyn goodmorning_services::bindings::traits::SerdeAny> {
        Box::new(BlueRenderDisplay {
            from: self.from.to_string_lossy().to_string(),
            to: self.to.to_string_lossy().to_string(),
            preset: self.preset.clone(),
        })
    }
}
