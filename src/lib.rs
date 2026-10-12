//! An unofficial, typed, async Rust client for the [Zalo Bot Platform](https://bot.zapps.me/docs/)
//! API. Not affiliated with or endorsed by Zalo or VNG.
//!
//! - Every API method as a typed request ([`methods`]) plus shortcuts on [`Bot`].
//! - Long polling as a `Stream` ([`Bot::updates`]) and webhook helpers ([`webhook`]), with a ready
//!   axum endpoint behind the `axum` feature.
//! - The token never appears in `Debug` output or error messages.
//!
//! ```no_run
//! use futures_util::StreamExt;
//! use zalo_bot::{Bot, Content, Token};
//!
//! # async fn run() -> zalo_bot::Result<()> {
//! let bot = Bot::new(Token::from_env("ZALO_BOT_TOKEN").expect("ZALO_BOT_TOKEN is set"));
//! let mut updates = std::pin::pin!(bot.updates(Default::default()));
//! while let Some(update) = updates.next().await {
//!     let Ok(update) = update else { continue };
//!     if let Content::Text(text) = update.content() {
//!         bot.send_message(update.chat_id(), format!("You said: {text}")).await?;
//!     }
//! }
//! # Ok(()) }
//! ```
//!
//! # TLS
//!
//! The default `default-tls` feature uses rustls with reqwest's default crypto provider. Without
//! it, install a rustls `CryptoProvider` before creating a [`Bot`] (for example
//! `rustls::crypto::ring::default_provider().install_default()`), or pass your own client to
//! [`Bot::with_client`].
//!
//! # What the API cannot do
//!
//! Zalo's Bot API has no file uploads (media is sent by URL), keyboards, message edits or
//! deletions, replies/threads, or group management. Groups are in beta: bots only receive messages
//! that mention them or reply to them.
#![cfg_attr(docsrs, feature(doc_cfg))]

mod client;
mod error;
pub mod methods;
mod polling;
pub mod text;
mod token;
mod types;
pub mod webhook;

pub use client::{API_BASE, Bot};
pub use error::{Error, Result};
pub use polling::PollOptions;
pub use token::Token;
pub use types::{
    BotInfo, Chat, ChatAction, ChatType, Content, EventName, Message, ParseMode, SentMessage,
    Style, TextStyle, Update, User, WebhookCheck, WebhookInfo, WebhookOutcome, WebhookSet,
};
