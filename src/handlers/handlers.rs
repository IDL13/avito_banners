use std::result;
use std::task::Poll;
use std::thread;

use axum::extract::Path;
use axum::middleware;
use axum::Json;
use futures::channel::mpsc;
use futures::TryStreamExt;
use serde_json::json;
use sqlx::Error;
use sqlx::Row;
use chrono::{DateTime, Utc};
use super::middleware::get_hash;
use super::ApiResponse::BannerForDeleteMany;
use super::ApiResponse::BannerRequestPost;
use super::ApiResponse::Content;
use super::ApiResponse::{ApiResponse, BannerId, Status400, Status500, UserBannerRequestForUser, UserBannerRequestAll, BannerResponsePost};
use super::Banner;
use crate::databases::postgres::Postgres;
use crate::databases::redis::Redis;
use super::middleware::new_token;

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
            is_active BOOLEAN,
            version INTEGER DEFAULT 1,
            created_at VARCHAR(512),
            updated_at VARCHAR(512))")
        .execute(&pool)
        .await?;

        sqlx::query("CREATE TABLE IF NOT EXISTS Versions (
            hash VARCHAR(512),
            banner_id INTEGER,
            tag_ids INTEGER[],
            feature_id INTEGER,
            title VARCHAR(512),
            text VARCHAR(512),
            url VARCHAR(512),
            is_active BOOLEAN,
            version INTEGER,
            created_at VARCHAR(512),
            updated_at VARCHAR(512))")
        .execute(&pool)
        .await?;

        sqlx::query("CREATE TABLE IF NOT EXISTS Admins_tokens (
            token VARCHAR(512))")
        .execute(&pool)
        .await?;

        sqlx::query("CREATE TABLE IF NOT EXISTS Users_tokens (
            token VARCHAR(512))")
        .execute(&pool)
        .await?;

    Ok(())
    }

    pub async fn user_banner(Json(json): Json<UserBannerRequestForUser>) -> ApiResponse {
        let db = Postgres::new().await;
        let mut cache = Redis::new();

        let pool = db.conn;

        if cache.check(json.tag_id, json.feature_id) {
            match cache.get(json.tag_id, json.feature_id) {
                Ok(strings) => {
                    let content = Content {
                        title: strings.0,
                        text: strings.1,
                        url: strings.2
                    };

                    return ApiResponse::JsonUserBanner(content)
                }
                Err(err) => {
                    println!("Err: {}", err);
                    return ApiResponse::JsonStatus500(Json(Status500{error: "Неизвестная ошибка сервера".to_string()}))
                }
            }
        };

        let mut result = sqlx::query("SELECT * FROM Banners
            WHERE $1 = ANY(tag_ids) AND feature_id = $2 AND is_active = $3")
        .bind(json.tag_id)
        .bind(json.feature_id)
        .bind(json.use_last_revision)
        .fetch(&pool);

        while let Some(row) = match result.try_next().await {
            Ok(row) => {row},
            Err(err) => {
                println!("{}", err);
                return ApiResponse::JsonStatus500(Json(Status500{error: "Неизвестная ошибка сервера".to_string()}));
            }
        } {
            let title: String = row.try_get("title").expect("Query dont have title");
            let text: String = row.try_get("text").expect("Query dont have text");
            let url: String = row.try_get("url").expect("Query dont have url");

            let content = Content {
                title,
                text, 
                url,
            };

            return ApiResponse::JsonUserBanner(content)
        };

        ApiResponse::JsonStatus404()
    }

    pub async fn banner_get(Json(json): Json<UserBannerRequestAll>) -> ApiResponse {
        let db = Postgres::new().await;

        let pool = db.conn;

        let mut ubr_vector: Vec<BannerResponsePost> = Vec::new();

        let mut result = sqlx::query("SELECT * FROM Banners
            WHERE feature_id = $1 AND $2 = ANY(tag_ids)
            LIMIT $3 OFFSET $4")
        .bind(json.feature_id)
        .bind(json.tag_id)
        .bind(json.limit)
        .bind(json.offset)
        .fetch(&pool);

        while let Some(row) = match result.try_next().await {
            Ok(row) => {row},
            Err(err) => {
                println!("{}", err);
                return ApiResponse::JsonStatus500(Json(Status500{error: "Неизвестная ошибка сервера".to_string()}));
            }
        } {
            let banner_id: i32 = row.try_get("banner_id").expect("Query dont have banner_id");
            let tag_ids: Vec<i32> = row.try_get("tag_ids").expect("Query dont have tag_id");
            let feature_id: i32 = row.try_get("feature_id").expect("Query dont have feature_id");
            let title: String = row.try_get("title").expect("Query dont have title");
            let text: String = row.try_get("text").expect("Query dont have text");
            let url: String = row.try_get("url").expect("Query dont have url");
            let is_active: bool = row.try_get("is_active").expect("Query dont have is_active");
            let created_at: String = row.try_get("created_at").expect("Query dont have created_at");
            let updated_at: String = row.try_get("updated_at").expect("Query dont have updated_at");

            let banner = BannerResponsePost {
                banner_id,
                tag_ids,
                feature_id,
                content: Content {
                    title,
                    text,
                    url,
                },
                is_active,
                created_at,
                updated_at
            };

            ubr_vector.push(banner);
        };

        ApiResponse::JsonBanner(ubr_vector)
    }

    pub async fn banner_post(Json(json): Json<BannerRequestPost>) -> ApiResponse {
        let db = Postgres::new().await;
        let mut cache = Redis::new();

        let pool = db.conn;

        for id in json.tag_ids.iter() {
            if !cache.check(id.clone(), json.feature_id.clone()) {
                let response = cache.set(
                    id.clone(),
                    json.feature_id,
                    vec![json.content.title.clone(), json.content.text.clone(), json.content.url.clone()]
                );
                println!("{}", response)
            }
        }
        let result: (i32,) = sqlx::query_as("INSERT INTO Banners
            (tag_ids, feature_id, title, text, url, is_active, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING banner_id")
        .bind(json.tag_ids)
        .bind(json.feature_id)
        .bind(json.content.title)
        .bind(json.content.text)
        .bind(json.content.url)
        .bind(json.is_active)
        .bind(Utc::now().to_string())
        .bind(Utc::now().to_string())
        .fetch_one(&pool).await.expect("Error from add Banner in db");

        ApiResponse::JsonBannerPost(result.0)
    }

    pub async fn banner_patch(Path(id): Path<i32>, Json(json): Json<BannerRequestPost>) -> ApiResponse {
        let db = Postgres::new().await;

        let pool = db.conn;

        let mut result = sqlx::query("SELECT * FROM Banners
            WHERE banner_id = $1")
        .bind(id)
        .fetch(&pool);

        while let Some(row) = match result.try_next().await {
            Ok(row) => {row},
            Err(err) => {
                println!("{}", err);
                return ApiResponse::JsonStatus500(Json(Status500{error: "Неизвестная ошибка сервера".to_string()}));
            }
        } {

            let banner_id: i32 = row.try_get("banner_id").expect("Query dont have banner_id");
            let tag_ids: Vec<i32> = row.try_get("tag_ids").expect("Query dont have tag_ids");
            let feature_id: i32 = row.try_get("feature_id").expect("Query dont have feature_id");
            let title: String = row.try_get("title").expect("Query dont have title");
            let text: String = row.try_get("text").expect("Query dont have text");
            let url: String = row.try_get("url").expect("Query dont have url");
            let is_active: bool = row.try_get("is_active").expect("Query dont have is_active");
            let version: i32 = row.try_get("version").expect("Query dont have version");
            let created_at: String = row.try_get("created_at").expect("Query dont have created_at");
            let updated_at: String = row.try_get("updated_at").expect("Query dont have updated_at");

            let hash = get_hash(banner_id, version).expect("Error from create hash for");

            let _: (i32,) = sqlx::query_as("INSERT INTO Version
            (hash banner_id, tag_ids, feature_id, title, text, url, is_active, version, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING banner_id")
            .bind(hash.as_str())
            .bind(banner_id)
            .bind(tag_ids)
            .bind(feature_id)
            .bind(title)
            .bind(text)
            .bind(url)
            .bind(is_active)
            .bind(version)
            .bind(created_at)
            .bind(updated_at)
            .fetch_one(&pool).await.expect("Error from add Banner in db");
        };

        let result = sqlx::query("UPDATE Banners
            SET  tag_ids = $1,
            feature_id = $2,
            title = $3,
            text = $4,
            url = $5,
            is_active = $6,
            updated_at = $7
            WHERE banner_id = $8")
        .bind(json.tag_ids)
        .bind(json.feature_id)
        .bind(json.content.title)
        .bind(json.content.text)
        .bind(json.content.url)
        .bind(json.is_active)
        .bind(Utc::now().to_string())
        .bind(id)
        .execute(&pool).await;

        match result {
            Ok(_) => {return ApiResponse::JsonStatus200()},
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

        ApiResponse::JsonStatus500(Json(Status500{error: "Неизвестная ошибка сервера".to_string()}))
    }

    pub async fn banner_delete(Path(id): Path<i32>) -> ApiResponse {
        let db = Postgres::new().await;

        let pool = db.conn;

        let result = sqlx::query("DELETE FROM Banners WHERE banner_id = $1")
        .bind(id)
        .execute(&pool).await;

        match result {
            Ok(_) => {return ApiResponse::JsonStatus200()},
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

        ApiResponse::JsonStatus500(Json(Status500{error: "Неизвестная ошибка сервера".to_string()}))
    }

    pub async fn new_token() -> ApiResponse {
        match new_token() {
            Ok(token) => {println!("{}", token)}
            Err(err) => {println!("{:?}", err)}
        }

        ApiResponse::JsonStatus200()
    }

    pub async fn banner_delete_many(Json(json): Json<BannerForDeleteMany>) -> ApiResponse {
        let db = Postgres::new().await;

        let pool = db.conn;

        let req: usize = json.tag_ids.len();
        let mut res: usize = 0;

        for tag in json.tag_ids.into_iter() {
            
            let result = sqlx::query("DELETE FROM Banners WHERE $1 = ANY(tag_ids)")
            .bind(tag)
            .execute(&pool).await;

            match result {
                Ok(_) => {
                    res += 1;
                },
                Err(err) => {
                    println!("Error in tag{} : {}", tag, err)
                },
            };
        }

        if req != res {
           return ApiResponse::JsonStatus500(Json(Status500{error: "Не полное удаление".to_string()}))
        } else {
            return ApiResponse::JsonStatus200();
        }
    }

}
