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
    JsonUserBanner(Content),
    JsonBanner(Vec<BannerResponsePost>),
    JsonBannerPost(i32),
    JsonStatus204(),
    JsonStatus200(),
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
            Self::JsonUserBanner(response) => {
                (
                    StatusCode::OK,
                    Json(response)
                ).into_response()
            },
            Self::JsonBanner(response) => {
                (
                    StatusCode::OK,
                    Json(response)
                ).into_response()
            }
            Self::JsonBannerPost(response) => {
                (
                    StatusCode::OK,
                    Json(json!({"banner_id": response}))
                ).into_response()
            }
            Self::JsonStatus204() => (StatusCode::OK,  Json(json!({"msg" : STATUS_204}))).into_response(),
            Self::JsonStatus200() => (StatusCode::OK, Json(json!({"msg": "Запрос успешно выполнен"}))).into_response(),
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
pub struct UserBannerRequestForUser {
    pub tag_id: i32,
    pub feature_id: i32,
    pub use_last_revision: bool,
}

#[derive(Serialize, Deserialize)]
pub struct UserBannerRequestAll {
    pub feature_id: i32,
    pub tag_id: i32,
    pub limit: i32,
    pub offset: i32,
}

#[derive(Serialize, Deserialize)]
pub struct BannerRequestPost {
    pub tag_ids: Vec<i32>,
    pub feature_id: i32,
    pub content: Content,
    pub is_active: bool,
}

#[derive(Serialize, Deserialize)]
pub struct BannerResponsePost {
    pub banner_id: i32,
    pub tag_ids: Vec<i32>,
    pub feature_id: i32,
    pub content: Content,
    pub is_active: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, Deserialize)]
pub struct Content {
    pub title: String,
    pub text: String,
    pub url: String,
}

#[derive(Serialize, Deserialize)]
pub struct BannerId {
    pub id: i32,
}