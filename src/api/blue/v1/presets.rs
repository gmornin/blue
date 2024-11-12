use actix_web::{get, HttpResponse};
use goodmorning_services::bindings::services::v1::V1Response;

use crate::values::{BLUE_CONFIG, PRESETS};

#[get("/presets")]
pub async fn presets() -> HttpResponse {
    HttpResponse::Ok().json(V1Response::BluePresets {
        default: BLUE_CONFIG.get().unwrap().default_preset.clone(),
        all: PRESETS.get().unwrap().clone(),
    })
}
