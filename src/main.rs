use std::char::ToLowercase;

use axum::{routing::{get, post, patch, delete}, Router};

mod handlers;
mod databases;

use handlers::Handlers;
use databases::postgres;

#[tokio::main]
async fn main() {

    // handlers::schema_db().await
    let handlers = Handlers::new().await;

    let app = Router::new()
        .route("/", get(handlers::healthiness_probe));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9000")
        .await
        .unwrap();

    println!("[SERVER STARTED] on 127.0.0.1:9000 port");

    axum::serve(listener, app).await.unwrap();
}
