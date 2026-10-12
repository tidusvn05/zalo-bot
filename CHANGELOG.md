# Changelog

All notable changes are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- `Bot` client for every Zalo Bot Platform API method: `getMe`, `getUpdates`, `setWebhook`,
  `deleteWebhook`, `getWebhookInfo`, `testWebhook`, `sendMessage` (with `parse_mode` and
  `text_styles`), `sendPhoto`, `sendSticker`, `sendVoice`, `sendChatAction`.
- Typed requests (`methods`) and models with forward-compatible enums.
- Long polling as a `Stream` (`Bot::updates`) with backoff and optional webhook removal.
- Webhook helpers (`verify_secret`, `parse_update`) and an axum endpoint (`axum` feature).
- Text helpers: `split_text`, `split_message`, `utf16_len`, `TextStyle::on`.
