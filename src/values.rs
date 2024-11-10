use std::{path::PathBuf, sync::OnceLock};

use goodmorning_services::{functions::parse_path, traits::ConfigTrait};

use crate::structs::BlueConfig;

pub static BLUE_CONFIG: OnceLock<BlueConfig> = OnceLock::new();

pub static PFP_DEFAULT: OnceLock<PathBuf> = OnceLock::new();

pub fn init() {
    let _ = BLUE_CONFIG.set(*BlueConfig::load().unwrap());
    let _ = PFP_DEFAULT.set(parse_path(BLUE_CONFIG.get().unwrap().pfp_default.clone()));
}
