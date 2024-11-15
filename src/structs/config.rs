use serde::{Deserialize, Serialize};

use goodmorning_services::{traits::ConfigTrait, LogOptions};

#[derive(Serialize, Deserialize, Clone)]
pub struct BlueConfig {
    #[serde(default = "pfp_default_default")]
    pub pfp_default: String,
    #[serde(default = "static_path_default")]
    pub static_path: String,
    #[serde(default = "log_default")]
    pub log: LogOptions,
    #[serde(default = "http_port_default")]
    pub port: u16,
    #[serde(default = "allow_create_default")]
    pub allow_create: bool,
    #[serde(default = "topbar_urls_default")]
    pub topbar_urls: Vec<UrlItem>,
    #[serde(default = "render_timeout_default")]
    pub render_timeout: u64,
    #[serde(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternate_pfp: Option<String>,
    #[serde(default = "default_preset_default")]
    pub default_preset: String,
}

fn allow_create_default() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UrlItem {
    pub label: String,
    pub url: String,
}

fn http_port_default() -> u16 {
    8080
}

impl Default for BlueConfig {
    fn default() -> Self {
        Self {
            static_path: static_path_default(),
            pfp_default: pfp_default_default(),
            // firejail_behavior: Default::default(),
            log: log_default(),
            port: 8080,
            allow_create: allow_create_default(),
            topbar_urls: topbar_urls_default(),
            alternate_pfp: None,
            default_preset: default_preset_default(),
            render_timeout: render_timeout_default(),
        }
    }
}

impl ConfigTrait for BlueConfig {
    const LABEL: &'static str = "blue";
}

fn static_path_default() -> String {
    "static".to_string()
}

fn pfp_default_default() -> String {
    "assets/pfp-default.svg".to_string()
}

fn log_default() -> LogOptions {
    LogOptions {
        loglabel: "gmblue".to_string(),
        termlogging: true,
        writelogging: true,
        term_log_level: goodmorning_services::LevelFilterSerde::Error,
        write_log_level: goodmorning_services::LevelFilterSerde::Debug,
    }
}

fn topbar_urls_default() -> Vec<UrlItem> {
    vec![
        UrlItem {
            url: "https://siriusmart.github.io/gm-services".to_string(),
            label: "API".to_string(),
        },
        UrlItem {
            url: "https://github.com/gmornin/gmt-server".to_string(),
            label: "Source code".to_string(),
        },
    ]
}

fn default_preset_default() -> String {
    "overworld.conf".to_string()
}

fn render_timeout_default() -> u64 {
    900
}
