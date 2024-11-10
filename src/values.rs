use std::{path::PathBuf, sync::OnceLock};

use goodmorning_services::{functions::parse_path, traits::ConfigTrait, SELF_ADDR};

use crate::structs::BlueConfig;

pub static BLUE_CONFIG: OnceLock<BlueConfig> = OnceLock::new();

pub static TOPBAR_URLS: OnceLock<String> = OnceLock::new();
pub static PFP_DEFAULT: OnceLock<PathBuf> = OnceLock::new();
pub static TOPBAR_LOGGEDOUT: OnceLock<String> = OnceLock::new();
pub static CSP_BASE: OnceLock<String> = OnceLock::new();

pub fn init() {
    let _ = BLUE_CONFIG.set(*BlueConfig::load().unwrap());
    let _ = PFP_DEFAULT.set(parse_path(BLUE_CONFIG.get().unwrap().pfp_default.clone()));

    CSP_BASE
        .set(format!(
            "script-src {}/static/scripts/",
            SELF_ADDR.get().unwrap()
        ))
        .unwrap();

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

    TOPBAR_LOGGEDOUT
        .set(format!(
            r#"<div id="top-bar">
      <div id="top-bar-left">
	<a href="/" id="top-bar-icon"><img src="/static/images/logo.webp" alt="" width="30"></a>
    {}
      </div>
      <div id="top-bar-right">
        <a href="/login" class="buttonlike buttonlike-hover" id="signin">Sign in</a>
        <a href="/login?type=new" class="buttonlike hover-dropshadow" id="top-bar-register"
          >Register</a
        >
      </div>
    </div>"#,
            TOPBAR_URLS.get().unwrap()
        ))
        .unwrap();
}
