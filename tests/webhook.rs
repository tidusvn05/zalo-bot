//! Webhook helpers and the axum endpoint.
#![allow(clippy::unwrap_used)] // test code: a panic is the failure report

use zalo_bot::webhook::{SECRET_HEADER, parse_update, verify_secret};

const BODY: &str = r#"{"ok":true,"result":{"event_name":"message.text.received","message":{
  "from":{"id":"u1","display_name":"Ted","is_bot":false},"chat":{"id":"c1","chat_type":"PRIVATE"},
  "text":"Hello","message_id":"m1","date":1750316131602}}}"#;

#[test]
fn secrets() {
    assert!(verify_secret(Some(b"12345678"), "12345678"));
    assert!(!verify_secret(Some(b"12345679"), "12345678"));
    assert!(!verify_secret(Some(b"1234567"), "12345678"));
    assert!(!verify_secret(None, "12345678"));
    assert!(!verify_secret(Some(b""), ""));
}

#[test]
fn bodies() {
    let update = parse_update(BODY.as_bytes()).unwrap();
    assert_eq!(update.chat_id(), "c1");
    let bare = serde_json::to_vec(&update).unwrap();
    assert_eq!(parse_update(&bare).unwrap(), update);
    assert!(parse_update(b"{}").is_err());
}

#[cfg(feature = "axum")]
#[tokio::test]
async fn axum_router() {
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let app = zalo_bot::webhook::router("12345678".into(), move |update| {
        let tx = tx.clone();
        async move {
            tx.send(update).unwrap();
        }
    });
    let request = |secret: Option<&str>, body: &str| {
        let mut r = Request::post("/");
        if let Some(s) = secret {
            r = r.header(SECRET_HEADER, s);
        }
        r.body(Body::from(body.to_string())).unwrap()
    };
    let status = |r: Request<Body>| {
        let app = app.clone();
        async move { app.oneshot(r).await.unwrap().status() }
    };
    assert_eq!(status(request(None, BODY)).await, StatusCode::UNAUTHORIZED);
    assert_eq!(
        status(request(Some("wrong-secret"), BODY)).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        status(request(Some("12345678"), "nope")).await,
        StatusCode::BAD_REQUEST
    );
    assert_eq!(
        status(request(Some("12345678"), BODY)).await,
        StatusCode::OK
    );
    let update = rx.recv().await.unwrap();
    assert_eq!(update.message.text.as_deref(), Some("Hello"));
}
