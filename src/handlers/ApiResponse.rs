use axum::{http::StatusCode, response::{ErrorResponse, IntoResponse, Response}, Json};
use serde_json::json;
use std::error;
use serde::{Serialize, Deserialize};

const STATUS_204: &str = "Баннер успешно удален";
const STATUS_401: &str = "Пользователь не авторизован";
const STATUS_403: &str = "Пользователь не имеет доступа";
const STATUS_404: &str = "Баннер для не найден";
const SERVER_START: &str = "Сервер запущен";

pub enum ApiResponse {
    JsonStr(),
    JsonCustom(String),
    JsonUserBanner(Vec<String>),
    JsonStatus204(),
    JsonStatus400(Json<Status400>),
    JsonStatus401(),
    JsonStatus403(),
    JsonStatus404(),
    JsonStatus500(Json<Status500>),
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        match self {
            Self::JsonStr() => (StatusCode::OK,  Json(json!({"msg" : SERVER_START}))).into_response(),
            Self::JsonCustom(msg) => (StatusCode::OK,  Json(json!({"msg" : msg}))).into_response(),
            Self::JsonUserBanner(response) => {
                (
                    StatusCode::OK,
                    Json(json!(response.join(",")))
                ).into_response()
            },
            Self::JsonStatus204() => (StatusCode::NO_CONTENT,  Json(json!({"msg" : STATUS_204}))).into_response(),
            Self::JsonStatus400(err) => (StatusCode::BAD_REQUEST,  err).into_response(),
            Self::JsonStatus401() => (StatusCode::OK,  Json(json!({"msg" : STATUS_401}))).into_response(),
            Self::JsonStatus403() => (StatusCode::OK,  Json(json!({"msg" : STATUS_403}))).into_response(),
            Self::JsonStatus404() => (StatusCode::OK,  Json(json!({"msg" : STATUS_404}))).into_response(),
            Self::JsonStatus500(err) => (StatusCode::INTERNAL_SERVER_ERROR,  err).into_response(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Status500 {
    pub error: String,
}

#[derive(Serialize, Deserialize)]
pub struct Status400 {
    pub error: String,
    pub type_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserBannerRequest {
    pub tag_id: i32,
    pub feature_id: i32,
    pub use_last_revision: bool,
}

#[derive(Serialize, Deserialize)]
pub struct BannerId {
    pub id: i32,
}