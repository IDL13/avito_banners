use axum::{extract::Request, http::request, middleware::Next, Json};
use futures::TryStreamExt;
use sha2::Sha256;
use sqlx::postgres::PgRow;
use sqlx::Error;
use sqlx::Row;
use hmac::{Hmac, Mac};
use jwt::{claims, SignWithKey};
use std::{collections::BTreeMap, time::SystemTime};
use super::ApiResponse::{ApiResponse, BannerId, Status400, Status500, UserBannerRequestForUser, UserBannerRequestAll};
use crate::postgres::Postgres;


pub struct Handlers {
}

impl Handlers {
    pub async fn healthiness_probe() -> ApiResponse {
        ApiResponse::JsonStr()
    }

    pub async fn schema_db() -> Result<(), sqlx::Error> {
        let db = Postgres::new().await;

        let pool = db.conn;

        sqlx::query("CREATE TABLE IF NOT EXISTS Banners (
            banner_id SERIAL PRIMARY KEY,
            tag_ids INTEGER[],
            feature_id INTEGER,
            title VARCHAR(512),
            text VARCHAR(512),
            url VARCHAR(512),
            is_active BIT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW())
        )")
        .execute(&pool)
        .await?;

        sqlx::query("CREATE TABLE IF NOT EXISTS Banners_last (
            banner_id SERIAL PRIMARY KEY,
            tag_ids INTEGER[],
            feature_id INTEGER,
            title VARCHAR(512),
            text VARCHAR(512),
            url VARCHAR(512),
            is_active BIT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW())
        )")
        .execute(&pool)
        .await?;

        sqlx::query("CREATE TABLE IF NOT EXISTS Admins_tokens (
            token VARCHAR(512),
        )")
        .execute(&pool)
        .await?;

        sqlx::query("CREATE TABLE IF NOT EXISTS Users_tokens (
            token VARCHAR(512),
        )")
        .execute(&pool)
        .await?;

    Ok(())
    }

    pub async fn user_banner(Json(json): Json<UserBannerRequestForUser>) -> ApiResponse {
        let db = Postgres::new().await;

        let pool = db.conn;

        let mut result = sqlx::query("SELECT * FROM Banners
            WHERE tag_id = $1,
            feature_id = $2,
            use_last_revision = $3")
        .bind(json.tag_id)
        .bind(json.feature_id)
        .bind(json.use_last_revision)
        .fetch(&pool);

        while let Some(row) = match result.try_next().await {
            Ok(row) => {row},
            Err(_) => {
                return ApiResponse::JsonStatus500(Json(Status500{error: "Неизвестная ошибка сервера".to_string()}));
            }
        } {
            let title: String = row.try_get("title").expect("Query dont have title");
            let text: String = row.try_get("text").expect("Query dont have text");
            let url: String = row.try_get("url").expect("Query dont have url");

            let json_string = format!(r#"{{ "title":{}, "text":{}, "url":{} }}"#, title, text, url).to_string();

            return ApiResponse::JsonUserBanner(json_string)
        };

        ApiResponse::JsonStatus404()
    }

    pub async fn banner_get(Json(json): Json<UserBannerRequestAll>) -> ApiResponse {
        let db = Postgres::new().await;

        let pool = db.conn;

        let mut ubr_vector: Vec<String> = Vec::new();

        let mut result = sqlx::query("SELECT * FROM Banners
            WHERE feature_id = $1,
            tag_id = $2
            LIMIT $3 OFFSET $4")
        .bind(json.feature_id)
        .bind(json.tag_id)
        .bind(json.limit)
        .bind(json.offset)
        .fetch(&pool);

        while let Some(row) = match result.try_next().await {
            Ok(row) => {row},
            Err(_) => {
                return ApiResponse::JsonStatus500(Json(Status500{error: "Неизвестная ошибка сервера".to_string()}));
            }
        } {
            let banner_id: i32 = row.try_get("banner_id").expect("Query dont have banner_id");
            let tag_id: Vec<i32> = row.try_get("tag_id").expect("Query dont have tag_id");
            let feature_id: i32 = row.try_get("feature_id").expect("Query dont have feature_id");
            let title: String = row.try_get("title").expect("Query dont have title");
            let text: String = row.try_get("text").expect("Query dont have text");
            let url: String = row.try_get("url").expect("Query dont have url");
            let is_active: bool = row.try_get("is_active").expect("Query dont have is_active");
            let created_at: String = row.try_get("created_at").expect("Query dont have created_at");
            let updated_at: String = row.try_get("updated_at").expect("Query dont have updated_at");

            let json_string = format!(r#"{{
                "banner_id":{},
                "tag_id":{:?},
                "feature_id":{},
                "content": {{
                    "title":{},
                    "text":{},
                    "url":{}
                }},
                "is_active":{},
                "created_at":{},
                "updated_at":{}
            }}"#,banner_id, tag_id, feature_id, title, text, url, is_active, created_at, updated_at).to_string();

            ubr_vector.push(json_string);
        };

        ApiResponse::JsonBanner(ubr_vector)
    }

    pub async fn banner_post() -> ApiResponse {
        ApiResponse::JsonStr()
    }

    pub async fn banner_patch() -> ApiResponse {
        ApiResponse::JsonStr()
    }

    pub async fn auth() -> ApiResponse {
        ApiResponse::JsonStr()
    }

    pub async fn banner_delete(Json(json): Json<BannerId>) -> ApiResponse {
        let db = Postgres::new().await;

        let pool = db.conn;

        let result = sqlx::query("DELETE FROM Banners WHERE id = $1")
        .bind(json.id)
        .execute(&pool).await;

        match result {
            Ok(_) => {},
            Err(err) => {
                match err {
                    Error::RowNotFound => {
                        ApiResponse::JsonStatus404();
                    }
                    Error::TypeNotFound { type_name } => {
                        ApiResponse::JsonStatus400(Json(Status400 { 
                            error: "Неизвестная ошибка сервера".to_string(),
                            type_name: type_name,
                        }));
                    },
                    _ => {
                        ApiResponse::JsonStatus500(Json(Status500{error: "Неизвестная ошибка сервера".to_string()}));
                    },
                }
            },
        };

        ApiResponse::JsonStatus204()
    }

}
