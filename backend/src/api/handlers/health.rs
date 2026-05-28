use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthCheck {
    success: bool,
    message: String,
}

pub async fn health_check() -> Json<HealthCheck> {
    Json(HealthCheck {
        success: true,
        message: "No worries, I am still healthy".to_string(),
    })
}
