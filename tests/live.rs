//! Opt-in tests against the real API.
//!
//! ```sh
//! ZALO_BOT_TOKEN=12345:abc cargo test --test live -- --ignored
//! ```
#![allow(clippy::unwrap_used)] // test code: a panic is the failure report

use zalo_bot::{Bot, Token};

#[tokio::test]
#[ignore = "calls the real Zalo API"]
async fn rejects_a_bogus_token() {
    let error = Bot::new("1:bogus").get_me().await.unwrap_err();
    assert!(error.is_unauthorized(), "{error}");
}

#[tokio::test]
#[ignore = "needs ZALO_BOT_TOKEN"]
async fn real_bot() {
    let bot = Bot::new(Token::from_env("ZALO_BOT_TOKEN").expect("set ZALO_BOT_TOKEN"));
    let me = bot.get_me().await.unwrap();
    assert_ne!(me.id, "");
    bot.get_webhook_info().await.unwrap();
}
