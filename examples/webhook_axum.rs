//! Echo bot over a webhook, with axum.
//!
//! Put it behind an HTTPS reverse proxy, then:
//!
//! ```sh
//! ZALO_BOT_TOKEN=12345:abc ZALO_WEBHOOK_SECRET=a-long-random-secret \
//!   ZALO_WEBHOOK_URL=https://bot.example.com/zalo \
//!   cargo run --example webhook_axum --features axum
//! ```

use zalo_bot::{Bot, Content, Token};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let bot = Bot::new(Token::from_env("ZALO_BOT_TOKEN")?);
    let secret = std::env::var("ZALO_WEBHOOK_SECRET")?;
    let url = std::env::var("ZALO_WEBHOOK_URL")?;

    let set = bot.set_webhook(&url, &secret).await?;
    if let Some(check) = set.verification {
        println!(
            "Zalo checked {}: {} {}",
            check.url, check.outcome, check.hint
        );
    }

    let replier = bot.clone();
    let zalo = zalo_bot::webhook::router(secret, move |update| {
        let bot = replier.clone();
        async move {
            if let Content::Text(text) = update.content()
                && let Err(error) = bot.send_message(update.chat_id(), text).await
            {
                eprintln!("reply failed: {error}");
            }
        }
    });
    let app = axum::Router::new().nest("/zalo", zalo);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    println!("listening on http://127.0.0.1:8080/zalo");
    axum::serve(listener, app).await?;
    Ok(())
}
