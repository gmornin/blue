use default_from_serde::SerdeDefault;
use goodmorning_services::traits::ConfigTrait;
use serde::{Deserialize, Serialize};
use serde_inline_default::serde_inline_default;

#[serde_inline_default]
#[derive(Serialize, Deserialize, Clone, SerdeDefault)]
pub struct BlueConfig {
    #[serde_inline_default("static".to_string())]
    pub static_path: String,
    #[serde_inline_default(8080)]
    pub port: u16,
    #[serde_inline_default(true)]
    pub allow_create: bool,
}

impl ConfigTrait for BlueConfig {
    const LABEL: &'static str = "blue";
}
