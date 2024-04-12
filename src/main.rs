use std::char::ToLowercase;

use axum::{middleware, routing::{delete, get, patch, post}, Router};

mod handlers;
mod databases;

use databases::postgres;
use databases::redis::Redis;
use handlers::Handlers;

use handlers::{jwt_for_admin, jwt_for_user};

#[tokio::main]
async fn main() {
    Handlers::schema_db().await.expect("Error creating database");

    let app = Router::new()
        .route("/", get(Handlers::healthiness_probe))
        .route("/user_banner",get(Handlers::user_banner).layer(middleware::from_fn(jwt_for_admin)))
        .route("/banner", get(Handlers::banner_get)).layer(middleware::from_fn(jwt_for_admin))
        .route("/banner", post(Handlers::banner_post).layer(middleware::from_fn(jwt_for_admin)))
        .route("/banner/:id", patch(Handlers::banner_patch).layer(middleware::from_fn(jwt_for_admin)))
        .route("/banner/:id", delete(Handlers::banner_delete).layer(middleware::from_fn(jwt_for_admin)))
        .route("/new_token", get(Handlers::new_token));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9000")
        .await
        .unwrap();

    println!("[SERVER STARTED] on 127.0.0.1:9000 port");

    axum::serve(listener, app).await.unwrap();
}
