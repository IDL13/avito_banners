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
        .connect("postgres://avito_banner:avito_banner@localhost:5555/avito_banner")
        .await.expect("Bad connection on database")
}