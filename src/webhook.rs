//! Receiving updates by webhook.
//!
//! Register the endpoint with [`Bot::set_webhook`](crate::Bot::set_webhook), then for every
//! request: check the secret header with [`verify_secret`], parse the body with
//! [`parse_update`], answer 200 at once and handle the update in the background (Zalo treats slow
//! answers as failures). The `axum` feature does all of this in [`router`].

use crate::types::Update;

/// The header carrying the secret given to `setWebhook`.
pub const SECRET_HEADER: &str = "X-Bot-Api-Secret-Token";

/// Compares the received header with the expected secret in constant time.
pub fn verify_secret(received: Option<&[u8]>, expected: &str) -> bool {
    let Some(received) = received else {
        return false;
    };
    let expected = expected.as_bytes();
    if received.len() != expected.len() || expected.is_empty() {
        return false;
    }
    received
        .iter()
        .zip(expected)
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

/// Parses a webhook body: `{"ok": true, "result": {…}}`, or a bare update.
pub fn parse_update(body: &[u8]) -> Result<Update, serde_json::Error> {
    #[derive(serde::Deserialize)]
    #[serde(untagged)]
    enum Body {
        Envelope { result: Update },
        Bare(Update),
    }
    Ok(match serde_json::from_slice(body)? {
        Body::Envelope { result } | Body::Bare(result) => result,
    })
}

#[cfg(feature = "axum")]
mod axum_router;

#[cfg(feature = "axum")]
#[cfg_attr(docsrs, doc(cfg(feature = "axum")))]
pub use axum_router::router;
