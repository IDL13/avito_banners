use axum::Json;
use futures::TryStreamExt;
use sqlx::Error;

use super::ApiResponse::{ApiResponse, BannerId, Status400, Status500};
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

    Ok(())
    }

    pub async fn user_banner() -> ApiResponse {
        ApiResponse::JsonStr()
    }

    pub async fn banner_get() -> ApiResponse {
        ApiResponse::JsonStr()
    }

    pub async fn banner_post() -> ApiResponse {
        ApiResponse::JsonStr()
    }

    pub async fn banner_patch() -> ApiResponse {
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
