use std::char::ToLowercase;

use axum::{middleware, routing::{delete, get, patch, post}, Router};

mod handlers;
mod databases;

use databases::postgres;
use handlers::Handlers;

use handlers::my_middleware;

#[tokio::main]
async fn main() {

    Handlers::schema_db().await.expect("Error creating database");

    let app = Router::new()
        .route("/", get(Handlers::healthiness_probe))
        .route("/user_banner",get(Handlers::user_banner).layer(middleware::from_fn(my_middleware)))
        .route("/banner", get(Handlers::banner_get))
        .route("/banner", post(Handlers::banner_post))
        .route("/banner/:id", patch(Handlers::banner_patch))
        .route("/banner/:id", delete(Handlers::banner_delete))
        .route("/new_token", get(Handlers::new_token));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9000")
        .await
        .unwrap();

    println!("[SERVER STARTED] on 127.0.0.1:9000 port");

    axum::serve(listener, app).await.unwrap();
}
