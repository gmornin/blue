use std::sync::OnceLock;

use goodmorning_services::traits::ConfigTrait;

use crate::config::BlueConfig;

pub static BLUE_CONFIG: OnceLock<BlueConfig> = OnceLock::new();

pub fn init() {
    let _ = BLUE_CONFIG.set(*BlueConfig::load().unwrap());
}
