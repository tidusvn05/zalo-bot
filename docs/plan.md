# zalo-bot plan

An unofficial, typed, async Rust client for the [Zalo Bot Platform](https://bot.zapps.me/docs/) API
(`bot-api.zaloplatforms.com`). It is the official, terms-of-service-compliant bot API, not the
reverse-engineered personal-account API.

## Why a new crate

| | zca-rs 0.1.0 | rs-bot-zalo 0.1.0 | zalo-bot |
|---|---|---|---|
| API | unofficial personal account (zca-js port, ToS risk) | official Bot API | official Bot API |
| Coverage of the 11 Bot API methods | n/a | 3 send methods + polling | all 11 |
| Typed models | responses are `serde_json::Value` | partial; non-text updates fail to parse | every request and response |
| Errors | not `#[non_exhaustive]` | `Box<dyn Error>` (not `Send`) | one `#[non_exhaustive]` `Error`, retry hints |
| Token safety | cookies/imei in `Debug` | token in `Debug` and in logged reqwest errors | redacted `Debug`, URLs stripped from errors |
| TLS | native-tls / OpenSSL | rustls (aws-lc) | rustls; crypto provider chosen by the app |
| Webhook | no | no | secret check + update parsing, optional axum handler |
| Tests | live-only for most | none | mock-server tests for every method |

## API facts that shape the design (docs of 2026-09-07)

- `POST https://bot-api.zaloplatforms.com/bot<TOKEN>/<method>` with a JSON body; the token is in the
  path, so request URLs are secret.
- Envelope `{ok, result?, description?, error_code?}`; **errors arrive with HTTP 200** and `ok: false`.
  `testWebhook` uses `errorCode` (camelCase) for its daily quota error 426; `getUpdates` answers 409
  while a webhook is set.
- `getUpdates` returns **one** update or nothing; there is no `offset`/`update_id`.
- No uploads (media by URL), no keyboards, no edit/delete, no reply-to, no group management.
- Text and captions: 1–2000 characters; `text_styles` offsets count UTF-16 units.
- Webhook header `X-Bot-Api-Secret-Token`; secret 8–256 characters; answer 2xx quickly.
- Groups are beta: the bot only receives mentions and replies to itself.

## Scope

### v0.1 — complete Bot API

1. `Bot` client: token as `Token` (redacted), configurable base URL and `reqwest::Client`.
2. Typed request structs with builders, one per method, all implementing `Request`
   (`const METHOD`, `type Response`); convenience methods on `Bot` for each:
   `get_me`, `get_update`, `set_webhook`, `delete_webhook`, `get_webhook_info`, `test_webhook`,
   `send_message`, `send_photo`, `send_sticker`, `send_voice`, `send_chat_action`.
3. Models: `Update`, `Event` (text, image, sticker, voice, unsupported, unknown), `Message`, `User`,
   `Chat`, `ChatType`, `BotInfo`, `WebhookInfo`, `WebhookCheck` with typed `WebhookOutcome`,
   `SentMessage`, `ParseMode`, `TextStyle`/`Style`, `ChatAction`. Unknown enum values never fail
   deserialization.
4. `Error`: `Api { code, description }` with helpers (`is_unauthorized`, `is_rate_limited`,
   `is_webhook_conflict`, `is_retryable`), `Invalid` (client-side validation), `Http` (URL removed),
   `Decode`.
5. Client-side validation: text/caption length, secret length, HTTPS webhook URL.
6. Polling: `bot.updates(opts)` → `Stream<Item = Result<Update>>` with backoff; optional automatic
   `deleteWebhook` on 409.
7. Webhook: `SECRET_HEADER`, constant-time `verify_secret`, `parse_update`; feature `axum`:
   a ready `Router` that verifies, answers 200 at once and hands updates to a handler.
8. Text helpers: `utf16_len`, `split_text` (2000-char chunks on line/word boundaries), `TextStyle`
   builder for ranges.
9. Features: `default-tls` (reqwest rustls with its default provider) on by default; without it the
   application installs a rustls `CryptoProvider` (e.g. ring) or passes its own client.

### v0.2 — ergonomics (driven by users)

- Dispatcher: `Router` of handlers by event and command prefix (`/start`), shared state.
- Retry middleware for 429/5xx with jitter; rate limiter.
- `tracing` spans per request (method name only, never the URL).

### Later

- Zalo Official Account (OA) API behind an `oa` feature, if there is demand.

## Quality gates

Edition 2024, MSRV 1.89, `#![forbid(unsafe_code)]`, `missing_docs = deny`, clippy pedantic, no
`unwrap` outside tests, `cargo deny`, docs.rs with all features, examples compiled in CI, mock-server
tests (`wiremock`) for every method and for error envelopes, an opt-in live smoke test
(`ZALO_BOT_TOKEN`).

## Consumers

okbot's `okbot-channel-zalo` switches from its private client to `zalo-bot` once v0.1 exists.
