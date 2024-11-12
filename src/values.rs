use std::{ffi::OsStr, fs, path::PathBuf, sync::OnceLock};

use bluemap_singleserve::{Config, MasterConfig};
use goodmorning_services::{functions::parse_path, traits::ConfigTrait, SELF_ADDR};

use crate::structs::BlueConfig;

pub static BLUE_CONFIG: OnceLock<BlueConfig> = OnceLock::new();

pub static TOPBAR_URLS: OnceLock<String> = OnceLock::new();
pub static PFP_DEFAULT: OnceLock<PathBuf> = OnceLock::new();
pub static CSP_BASE: OnceLock<String> = OnceLock::new();
pub static PRESETS: OnceLock<Vec<String>> = OnceLock::new();

pub fn init() {
    let _ = BLUE_CONFIG.set(*BlueConfig::load().unwrap());
    let _ = PFP_DEFAULT.set(parse_path(BLUE_CONFIG.get().unwrap().pfp_default.clone()));

    CSP_BASE
        .set(format!(
            "script-src {}/static/scripts/",
            SELF_ADDR.get().unwrap()
        ))
        .unwrap();

    let mut presets = fs::read_dir(&MasterConfig::get().templates)
        .unwrap()
        .filter_map(|entry| {
            if entry.as_ref().unwrap().path().extension() == Some(OsStr::new("conf")) {
                Some(
                    entry
                        .unwrap()
                        .path()
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string(),
                )
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    presets.sort();

    PRESETS.set(presets).unwrap();

    TOPBAR_URLS
        .set(
            BLUE_CONFIG
                .get()
                .unwrap()
                .topbar_urls
                .iter()
                .map(|item| {
                    format!(
                        r#"<a href="{}" class="top-bar-link">{}</a>"#,
                        html_escape::encode_safe(&item.url),
                        html_escape::encode_safe(&item.label)
                    )
                })
                .collect::<Vec<_>>()
                .join(""),
        )
        .unwrap();
}
