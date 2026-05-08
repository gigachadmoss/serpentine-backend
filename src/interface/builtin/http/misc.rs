use actix_web::{get, web};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
struct TimeResponse {
    time: i64,
}

#[get("/api/time/raw")]
pub async fn time_raw() -> web::Json<TimeResponse> {
    web::Json(TimeResponse {
        time: crate::misc_util::current_time_secs(),
    })
}

#[get("/api/time/raw_micros")]
pub async fn time_raw_micros() -> web::Json<TimeResponse> {
    web::Json(TimeResponse {
        time: crate::misc_util::current_time_micros(),
    })
}
