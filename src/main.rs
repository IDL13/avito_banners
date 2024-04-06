use std::char::ToLowercase;

use axum::{routing::{get, post, patch, delete}, Router};

mod handlers;
mod databases;

use databases::postgres;
use handlers::Handlers;

#[tokio::main]
async fn main() {

    Handlers::schema_db().await.expect("Error creating database");

    let app = Router::new()
        .route("/", get(Handlers::healthiness_probe))
        .route("/user_banner",get(Handlers::user_banner))
        .route("/banner", get(Handlers::banner_get))
        .route("/banner", post(Handlers::banner_post))
        .route("/banner/:id", patch(Handlers::banner_patch))
        .route("/banner/:id", delete(Handlers::banner_delete));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9000")
        .await
        .unwrap();

    println!("[SERVER STARTED] on 127.0.0.1:9000 port");

    axum::serve(listener, app).await.unwrap();
}
