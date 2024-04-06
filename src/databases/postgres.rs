use sqlx::{postgres::PgPoolOptions, PgPool};
use std::error;

pub struct Postgres {
    pub conn: PgPool,
}

impl Postgres {
    pub async fn new() -> Self {
        Self {
            conn: connection().await,
        }
    }
}

async fn connection() -> PgPool {
    PgPoolOptions::new()
        .max_connections(100)
        .connect("postgres://banned:banned@localhost:5555/banned")
        .await.expect("Bad connection on database")
}