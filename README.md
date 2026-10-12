# zalo-bot

[![CI](https://github.com/tidusvn05/zalo-bot/actions/workflows/ci.yml/badge.svg)](https://github.com/tidusvn05/zalo-bot/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/zalo-bot)](https://docs.rs/zalo-bot)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![MSRV 1.89](https://img.shields.io/badge/rustc-1.89%2B-orange.svg)](Cargo.toml)

An **unofficial**, typed, async Rust client for the **[Zalo Bot Platform](https://bot.zapps.me/docs/)**
API: send messages, receive them by long polling or webhook.

> zalo-bot is a community project. It is not made, endorsed or supported by Zalo or VNG; "Zalo" is
> their trademark.

It talks to the official Bot API (`bot-api.zaloplatforms.com`), the one Zalo supports for bots, not a
reverse-engineered personal-account API.

## Features

- **The whole Bot API**: all 11 methods, typed requests and responses.
- **Long polling as a `Stream`** with backoff, and optional removal of a webhook that blocks it.
- **Webhooks**: constant-time secret check, update parsing, and a ready axum endpoint (`axum` feature).
- **Safe with secrets**: the token never appears in `Debug` output or error messages (it is part of
  every request URL, so URLs are stripped from errors).
- **Forward compatible**: unknown event names, chat types and webhook outcomes deserialize into
  `Other(..)` instead of failing.
- **Validation before sending**: text and caption length (1–2000 characters), webhook URL and secret.
- **Helpers**: splitting long text into 2000-character messages, UTF-16 offsets for `text_styles`.
- rustls only (no OpenSSL); bring your own crypto provider or `reqwest::Client` if you need to.

## Install

```toml
[dependencies]
zalo-bot = { git = "https://github.com/tidusvn05/zalo-bot" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
futures-util = "0.3"
```

Get a bot token from the **Zalo Bot Creator** mini app in Zalo (it looks like `12345689:abc-xyz`).

## Echo bot (long polling)

```rust,no_run
use futures_util::StreamExt;
use zalo_bot::{Bot, Content, PollOptions, Token};

#[tokio::main]
async fn main() -> zalo_bot::Result<()> {
    let bot = Bot::new(Token::from_env("ZALO_BOT_TOKEN").expect("set ZALO_BOT_TOKEN"));
    let mut updates = std::pin::pin!(bot.updates(PollOptions::default()));
    while let Some(update) = updates.next().await {
        let Ok(update) = update else { continue };
        if let Content::Text(text) = update.content() {
            bot.send_markdown(update.chat_id(), format!("Bạn vừa nói: **{text}**")).await?;
        }
    }
    Ok(())
}
```

## Webhook (axum)

```rust,ignore
let bot = Bot::new(token);
bot.set_webhook("https://bot.example.com/zalo", &secret).await?;

let replier = bot.clone();
let app = axum::Router::new().nest(
    "/zalo",
    zalo_bot::webhook::router(secret, move |update| {
        let bot = replier.clone();
        async move {
            let _ = bot.send_message(update.chat_id(), "Got it").await;
        }
    }),
);
```

Without axum, use `webhook::verify_secret` and `webhook::parse_update` in any framework. Answer 200
quickly and handle the update in the background: Zalo treats slow answers as failures.

More in [`examples/`](examples/).

## API coverage

| Method | zalo-bot | Notes |
|---|---|---|
| `getMe` | `Bot::get_me` | |
| `getUpdates` | `Bot::get_updates`, `Bot::updates` (stream) | at most one update per call; no offset |
| `setWebhook` | `Bot::set_webhook` | returns Zalo's immediate endpoint check |
| `deleteWebhook` | `Bot::delete_webhook` | |
| `getWebhookInfo` | `Bot::get_webhook_info` | |
| `testWebhook` | `Bot::test_webhook` | daily limit (error 426) |
| `sendMessage` | `Bot::send_message`, `Bot::send_markdown`, `methods::SendMessage` | `parse_mode` (markdown/html), `text_styles` |
| `sendPhoto` | `Bot::send_photo` | by URL, optional caption |
| `sendSticker` | `Bot::send_sticker` | sticker id |
| `sendVoice` | `Bot::send_voice` | `.aac` URL, private chats only |
| `sendChatAction` | `Bot::send_chat_action` | `typing`, `upload_photo` |

Events: `message.text.received`, `message.image.received`, `message.sticker.received`,
`message.voice.received`, `message.unsupported.received` (see `Update::content`).

The Bot API itself has **no** file uploads, keyboards or buttons, message edits or deletions,
reply-to/threads, or group management, so neither does this crate. Groups are in beta: bots only receive
messages that mention them or reply to them.

## Compared with other crates

| | zalo-bot | [rs-bot-zalo](https://crates.io/crates/rs-bot-zalo) 0.1 | [zca-rs](https://crates.io/crates/zca-rs) 0.1 |
|---|---|---|---|
| API | official Bot API | official Bot API | unofficial personal account (ToS risk) |
| Bot API methods | 11 / 11 | 3 send methods + polling | n/a |
| Webhook | yes | no | no |
| Typed responses | yes | partly | no (`serde_json::Value`) |
| Token kept out of logs | yes | no | n/a (cookies in `Debug`) |
| TLS | rustls | rustls | native-tls/OpenSSL |
| Tests | mock-server tests for every method | none | mostly live tests |

## TLS

The default `default-tls` feature enables rustls with reqwest's default crypto provider. To use another
provider (for example ring), turn default features off and install it once at start-up:

```toml
zalo-bot = { git = "https://github.com/tidusvn05/zalo-bot", default-features = false }
```

```rust,ignore
rustls::crypto::ring::default_provider().install_default().ok();
let bot = zalo_bot::Bot::new(token);
```

Or pass your own client to `Bot::with_client`.

## Minimum supported Rust version

1.89. Raising it is a minor-version change.

## Contributing

Issues and pull requests are welcome; see [CONTRIBUTING.md](CONTRIBUTING.md). Report security issues
privately, see [SECURITY.md](SECURITY.md).

This project is not affiliated with Zalo or VNG.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT license](LICENSE-MIT)
at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
zalo-bot by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
