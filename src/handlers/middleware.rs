use axum::{
    body::Body, http::{response, HeaderMap, Request, StatusCode}, middleware::Next, response::{IntoResponse, Response}, Json
};
use sha2::Sha256;
use hmac::{Hmac, Mac};
use jwt::{claims, SignWithKey};
use std::{collections::BTreeMap, time::SystemTime};

use crate::handlers::ApiResponse::*;
use serde_json::json;

pub struct Middleware {

}

impl Middleware {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn add_jwt_token(mut request: Request<Body>, next: Next) -> impl IntoResponse {
        let b_string = SystemTime::now().elapsed().unwrap().as_secs_f32().to_be_bytes();

        let key: Hmac<Sha256> = Hmac::new_from_slice(&b_string).expect("Error from Sha256 algorithm");

        let mut claims = BTreeMap::new(); 
        claims.insert("sub", "someone");
        let token_str = claims.sign_with_key(&key);

        match token_str {
            Ok(token) => {
                let headers = request.headers_mut();
                headers.insert(axum::http::header::AUTHORIZATION, axum::http::header::HeaderValue::from_str(&token).unwrap());

                let response = next.run(request).await;

                response
            },
            Err(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR,  Json(json!({"msg": "Неизвестная ошибка"}))).into_response()
            },
        };
    }
}