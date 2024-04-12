use std::collections::BTreeMap;
use std::error;
use axum::{
    extract::Request, handler::Handler, http::{self, HeaderMap}, middleware::{self, Next}, response::Response, routing::get, Router
};
use futures::TryStreamExt;
use std::iter::zip;
use chrono::Utc;
use hmac::{Hmac, Mac};
use jwt::{token, Header, SignWithKey};
use reqwest::{header::AUTHORIZATION, StatusCode};
use sha2::Sha256;

use crate::handlers::ApiResponse::*;
use serde_json::json;

use crate::postgres::Postgres;

use rand::Rng;

pub async fn jwt_for_admin(headers: HeaderMap, request: Request, next: Next) -> Result<Response, ApiResponse>{
    match get_token(&headers) {
        Some(token) if admin_token_is_valid(token).await => {
            let response = next.run(request).await;
            Ok(response)
        }

        _ => {
            Err(ApiResponse::JsonStatus401())
        }
    }
}

pub async fn jwt_for_user(headers: HeaderMap, request: Request, next: Next) -> Result<Response, ApiResponse>{
    match get_token(&headers) {
        Some(token) if user_token_is_valid(token).await => {
            let response = next.run(request).await;
            Ok(response)
        }

        _ => {
            Err(ApiResponse::JsonStatus401())
        }
    }
}

fn get_token(headers: &HeaderMap) -> Option<&str> {
    match headers.get(AUTHORIZATION) {
        Some(token) => match token.to_str() {
            Ok(token_str) => Some(token_str),
            Err(err) => {
                eprintln!("Error parsing token: {}", err);
                None
            }
        },
        None => {
            eprintln!("Authorization header not found");
            None
        }
    }
}

async fn admin_token_is_valid(token: &str) -> bool {
    let db = Postgres::new().await;

    let pool = db.conn;

    let mut result = sqlx::query("SELECT * FROM Admins_tokens
        WHERE token = $1")
    .bind(token)
    .fetch(&pool);

    match result.try_next().await {
        Ok(row) => {
            match row {
                Some(_) => return true,
                None => return false,
            };
        },
        Err(err) => {
            println!("{}", err);
            return false
        }
    };
}

async fn user_token_is_valid(token: &str) -> bool {
    let db = Postgres::new().await;

    let pool = db.conn;

    let mut result = sqlx::query("SELECT * FROM Users_tokens
        WHERE token = $1")
    .bind(token)
    .fetch(&pool);

    match result.try_next().await {
        Ok(row) => {
            match row {
                Some(_) => return true,
                None => return false,
            };
        },
        Err(err) => {
            println!("{}", err);
            return false
        }
    };
}

pub fn new_token() -> Result<String, Box<dyn error::Error>> {
    let SALT1 = "JL#K@J4lj23l4kj23ljj";
    let SALT2 = "j3kjl3kj4v0293jv23;vlk2;l3;kv";
    let SALt3 = "lj3[,c2.xj0x4j4jx094x20j4vh,2c-hx2.3xj.0";

    let mut rng = rand::thread_rng();

    let salt = match rng.gen_range(0..10000) {
        0..=5000 => SALT1.as_bytes(), 
        5001..=10000 => SALT2.as_bytes(),
        _ => SALt3.as_bytes(),
    };

    let hash_key: Vec<u8> = [Utc::now().to_string().as_bytes(), salt].concat();

    let key: Hmac<Sha256> = Hmac::new_from_slice(&hash_key)?;
    let mut claims = BTreeMap::new();
    claims.insert("sub", "someone");
    let token_str = claims.sign_with_key(&key)?;

    Ok(token_str)
}

