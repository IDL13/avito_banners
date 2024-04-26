use axum::{http::{response, StatusCode}, middleware, routing::{delete, get, patch, post}, Json, Router};

mod handlers;
mod databases;

use axum_test::TestServer;
use databases::postgres;
use handlers::Handlers;
use handlers::{Content, UserBannerRequestForUser, BannerResponsePost};

use handlers::{jwt_for_admin, jwt_for_user};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tower_cookies::CookieManagerLayer;

#[tokio::main]
async fn main() {
    Handlers::schema_db().await.expect("Error creating database");

    let app = Router::new()
        .route("/", get(Handlers::healthiness_probe))
        .route("/user_banner",get(Handlers::user_banner).layer(middleware::from_fn(jwt_for_admin)))
        .route("/banner", get(Handlers::banner_get)).layer(middleware::from_fn(jwt_for_user))
        .route("/banner", post(Handlers::banner_post).layer(middleware::from_fn(jwt_for_admin)))
        .route("/banner/:id", patch(Handlers::banner_patch).layer(middleware::from_fn(jwt_for_admin)))
        .route("/banner/:id", delete(Handlers::banner_delete).layer(middleware::from_fn(jwt_for_admin)))
        .route("/banner", delete(Handlers::banner_delete_many))
        .route("/new_token", get(Handlers::new_token));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:9000")
        .await
        .unwrap();

    println!("[SERVER STARTED] on 127.0.0.1:9000 port");

    axum::serve(listener, app).await.unwrap();
}

#[tokio::test]
async fn it_get_user_banner() {

    let app = Router::new()
    .route("/user_banner",get(Handlers::user_banner));

    let server = TestServer::new(app).expect("Error from new test server");

    let response = server.get(&"/user_banner").json(&json!({
        "tag_id": 3,
        "feature_id": 1,
        "use_last_revision": true
    })).await
    .json::<Content>();

    let true_ans = Content {
        title: "some_title".to_string(),
        text: "some_text".to_string(),
        url: "some_url".to_string(),
    };

    assert_eq!(response.text, true_ans.text);
    assert_eq!(response.title, true_ans.title);
    assert_eq!(response.url, true_ans.url);
}

#[tokio::test]
async fn it_get_banner() {

    let app = Router::new()
    .route("/banner",get(Handlers::banner_get));

    let server = TestServer::new(app).expect("Error from new test server");

    let response = server.get(&"/banner").json(&json!({
        "feature_id": 0,
        "tag_id": 0,
        "limit": 10,
        "offset":0
    })).await
    .json::<Vec<BannerResponsePost>>();

    let true_ans = BannerResponsePost {
        banner_id: 1,
        tag_ids: [0].to_vec(),
        feature_id: 0,
        content: Content {
            title: "some_title".to_string(),
            text: "some_text".to_string(),
            url: "some_url".to_string()
            },
        is_active: true,
        created_at: "2024-04-12 11:04:05.529046752 UTC".to_string(),
        updated_at: "2024-04-12 11:04:05.529054977 UTC".to_string()
    };

    assert_eq!(response[0].banner_id, true_ans.banner_id);
    assert_eq!(response[0].tag_ids, true_ans.tag_ids);
    assert_eq!(response[0].feature_id, true_ans.feature_id);
}

#[tokio::test]
async fn it_post_banner() {

    #[derive(Serialize, Deserialize)]
    struct Ans {
        banner_id: i32
    }

    let app = Router::new()
    .route("/banner",post(Handlers::banner_post));

    let server = TestServer::new(app).expect("Error from new test server");

    let response = server.post(&"/banner").json(&json!({
        "tag_ids": [
          5, 4, 2
        ],
        "feature_id": 1,
        "content": {
          "title": "some_title",
          "text": "some_text",
          "url": "some_url"
        },
        "is_active": true
      })).await
    .json::<Ans>();

    let true_ans = 8;

    assert_eq!(response.banner_id, true_ans);
}

#[tokio::test]
async fn it_patch_banner() {

    let app = Router::new()
    .route("/banner/:id", patch(Handlers::banner_patch));

    let server = TestServer::new(app).expect("Error from new test server");

    let response = server.patch(&"/banner/8").json(&json!({
        "tag_ids": [
          0, 1
        ],
        "feature_id": 0,
        "content": {
          "title": "some_title",
          "text": "some_text",
          "url": "some_url"
        },
        "is_active": true
      })).await.status_code();

    assert_eq!(response, StatusCode::OK);
}

#[tokio::test]
async fn it_delete_banner() {

    let app = Router::new()
    .route("/banner/:id", delete(Handlers::banner_delete));

    let server = TestServer::new(app).expect("Error from new test server");

    let response = server.delete("/banner/8").await.status_code();

    assert_eq!(response, StatusCode::OK);
}