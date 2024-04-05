use super::ApiResponse::ApiResponse;
use crate::postgres::Postgres;

pub struct Handlers {
    db_conn: Postgres,
}

impl Handlers {
    pub async fn new() -> Self {
        Self {
            db_conn: Postgres::new().await,
        }
    }

    pub async fn healthiness_probe() -> ApiResponse {
        ApiResponse::JsonStr()
    }
}

pub async fn healthiness_probe() -> ApiResponse {
    ApiResponse::JsonStr()
}
