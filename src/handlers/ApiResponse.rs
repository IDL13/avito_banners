use axum::{http::StatusCode, response::{ErrorResponse, IntoResponse, Response}, Json};
use serde_json::json;
use std::error;
use serde::{Serialize, Deserialize};

const STATUS_400: &str = "Некорректные данные";
const STATUS_401: &str = "Пользователь не авторизован";
const STATUS_403: &str = "Пользователь не имеет доступа";
const STATUS_404: &str = "Баннер для не найден";
const STATUS_500: &str = "Внутренняя ошибка сервера";
const SERVER_START: &str = "Сервер запущен";

pub enum ApiResponse {
    JsonStr(),
    JsonUserBanner(UserBanner),
    JsonStatus400(Box<dyn error::Error>),
    JsonStatus401(),
    JsonStatus403(),
    JsonStatus404(),
    JsonStatus500(Box<dyn error::Error>),
}

impl IntoResponse for ApiResponse {
    fn into_response(self) -> Response {
        match self {
            Self::JsonStr() => (StatusCode::OK,  Json(json!({"msg" : SERVER_START}))).into_response(),
            Self::JsonUserBanner(response) => {
                (
                    StatusCode::OK,
                    Json(json!({"title" : response.title, "text" : response.text, "url" : response.url}))
                ).into_response()
            },
            Self::JsonStatus400(err) => (StatusCode::OK,  Json(json!({"msg" : STATUS_400, "error": err.to_string()}))).into_response(),
            Self::JsonStatus401() => (StatusCode::OK,  Json(json!({"msg" : STATUS_401}))).into_response(),
            Self::JsonStatus403() => (StatusCode::OK,  Json(json!({"msg" : STATUS_403}))).into_response(),
            Self::JsonStatus404() => (StatusCode::OK,  Json(json!({"msg" : STATUS_404}))).into_response(),
            Self::JsonStatus500(err) => (StatusCode::OK,  Json(json!({"msg" : STATUS_500, "error": err.to_string()}))).into_response(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct UserBanner {
    pub title: String,
    pub text: String,
    pub url: String,
}