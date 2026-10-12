use std::future::Future;
use std::sync::Arc;

use axum::Router;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::routing::post;

use super::{SECRET_HEADER, parse_update, verify_secret};
use crate::types::Update;

/// An axum router answering `POST /`: it checks the secret (401 when wrong), parses the update
/// (400 when invalid), answers 200 at once and runs `handler` in a new task.
///
/// Mount it where `setWebhook` points, e.g. `Router::new().nest("/zalo", router(secret, handler))`.
///
/// ```no_run
/// # async fn run() {
/// let app = zalo_bot::webhook::router("my-secret-token".to_string(), |update| async move {
///     println!("{:?}", update.content());
/// });
/// let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await.unwrap();
/// axum::serve(listener, app).await.unwrap();
/// # }
/// ```
pub fn router<F, Fut>(secret: String, handler: F) -> Router
where
    F: Fn(Update) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let state = Arc::new((secret, handler));
    Router::new()
        .route("/", post(receive::<F, Fut>))
        .with_state(state)
}

async fn receive<F, Fut>(
    State(state): State<Arc<(String, F)>>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode
where
    F: Fn(Update) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static,
{
    let (secret, handler) = &*state;
    let received = headers
        .get(SECRET_HEADER)
        .map(axum::http::HeaderValue::as_bytes);
    if !verify_secret(received, secret) {
        return StatusCode::UNAUTHORIZED;
    }
    match parse_update(&body) {
        Ok(update) => {
            tokio::spawn(handler(update));
            StatusCode::OK
        }
        Err(_) => StatusCode::BAD_REQUEST,
    }
}
