//! Echo bot over long polling.
//!
//! ```sh
//! ZALO_BOT_TOKEN=12345:abc cargo run --example echo
//! ```

use futures_util::StreamExt;
use zalo_bot::{Bot, ChatAction, Content, PollOptions, Token};

#[tokio::main]
async fn main() -> zalo_bot::Result<()> {
    let token = Token::from_env("ZALO_BOT_TOKEN").expect("set ZALO_BOT_TOKEN");
    let bot = Bot::new(token);
    let me = bot.get_me().await?;
    println!("{} is listening (Ctrl-C to stop)", me.account_name);

    let mut updates = std::pin::pin!(bot.updates(PollOptions::default()));
    while let Some(update) = updates.next().await {
        let update = match update {
            Ok(update) => update,
            Err(error) if error.is_retryable() => {
                eprintln!("retrying: {error}");
                continue;
            }
            Err(error) => return Err(error),
        };
        let chat = update.chat_id().to_string();
        bot.send_chat_action(&chat, ChatAction::Typing).await?;
        match update.content() {
            Content::Text(text) => {
                bot.send_markdown(&chat, format!("Bạn vừa nói: **{text}**"))
                    .await?;
            }
            Content::Image { url, .. } => {
                bot.send_photo(&chat, url, Some("Ảnh của bạn".into()))
                    .await?;
            }
            Content::Sticker { id, .. } => {
                bot.send_sticker(&chat, id).await?;
            }
            _ => {
                bot.send_message(&chat, "Mình chưa hiểu loại tin nhắn này.")
                    .await?;
            }
        }
    }
    Ok(())
}
