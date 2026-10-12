//! The API client.

use std::time::Duration;

use serde_json::Value;

use crate::error::{Error, Result};
use crate::methods::{
    DeleteWebhook, GetMe, GetUpdates, GetWebhookInfo, Request, SendChatAction, SendMessage,
    SendPhoto, SendSticker, SendVoice, SetWebhook, TestWebhook,
};
use crate::token::Token;
use crate::types::{
    BotInfo, ChatAction, ParseMode, SentMessage, Update, WebhookCheck, WebhookInfo, WebhookSet,
};

/// The production API.
pub const API_BASE: &str = "https://bot-api.zaloplatforms.com";

/// Time allowed for a request that is not a long poll.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

/// A Zalo bot: cheap to clone, safe to share between tasks.
///
/// ```no_run
/// # async fn run() -> zalo_bot::Result<()> {
/// let bot = zalo_bot::Bot::new(zalo_bot::Token::from_env("ZALO_BOT_TOKEN").expect("token"));
/// let me = bot.get_me().await?;
/// println!("logged in as {}", me.account_name);
/// # Ok(()) }
/// ```
#[derive(Clone)]
pub struct Bot {
    http: reqwest::Client,
    base: String,
    token: Token,
}

impl std::fmt::Debug for Bot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Bot")
            .field("token", &self.token)
            .field("base", &self.base)
            .finish_non_exhaustive()
    }
}

impl Bot {
    /// A bot using the production API and a new HTTP client.
    ///
    /// # Panics
    /// Without the `default-tls` feature, when no rustls `CryptoProvider` is installed (install
    /// one first, or use [`Bot::with_client`]).
    pub fn new(token: impl Into<Token>) -> Self {
        let http = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .user_agent(concat!("zalo-bot/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("the HTTP client builds: TLS backend available");
        Self::with_client(token, http)
    }

    /// A bot using your own `reqwest::Client` (proxy, timeouts, TLS setup).
    pub fn with_client(token: impl Into<Token>, http: reqwest::Client) -> Self {
        Self {
            http,
            base: API_BASE.to_string(),
            token: token.into(),
        }
    }

    /// Uses another API base URL (a mock server in tests, or the legacy `https://bot-api.zapps.me`).
    pub fn with_base_url(mut self, base: impl Into<String>) -> Self {
        self.base = base.into().trim_end_matches('/').to_string();
        self
    }

    /// The token.
    pub fn token(&self) -> &Token {
        &self.token
    }

    /// Sends any request.
    pub async fn send<R: Request>(&self, request: R) -> Result<R::Response> {
        self.send_with_timeout(&request, REQUEST_TIMEOUT).await
    }

    async fn send_with_timeout<R: Request>(
        &self,
        request: &R,
        timeout: Duration,
    ) -> Result<R::Response> {
        request.validate()?;
        let url = format!("{}/bot{}/{}", self.base, self.token.expose(), R::METHOD);
        let response = self
            .http
            .post(url)
            .timeout(timeout)
            .json(request)
            .send()
            .await?;
        let status = response.status();
        let body = response.bytes().await?;
        let envelope: Envelope = serde_json::from_slice(&body).map_err(|e| Error::Decode {
            method: R::METHOD,
            message: format!("HTTP {status}: {e}"),
        })?;
        if !envelope.ok {
            return Err(Error::Api {
                code: envelope
                    .error_code
                    .unwrap_or_else(|| i64::from(status.as_u16())),
                description: envelope.description.unwrap_or_default(),
            });
        }
        R::decode(envelope.result).map_err(|e| Error::Decode {
            method: R::METHOD,
            message: e.to_string(),
        })
    }

    /// `getMe`.
    pub async fn get_me(&self) -> Result<BotInfo> {
        self.send(GetMe {}).await
    }

    /// `getUpdates`: waits up to `timeout` for events (at most one in practice). An empty list
    /// means none arrived in time.
    pub async fn get_updates(&self, timeout: Duration) -> Result<Vec<Update>> {
        let request = GetUpdates {
            timeout: timeout.as_secs(),
        };
        match self
            .send_with_timeout(&request, timeout + Duration::from_secs(10))
            .await
        {
            // Some API versions report "nothing within the timeout" as 408.
            Err(Error::Api { code: 408, .. }) => Ok(Vec::new()),
            other => other,
        }
    }

    /// `setWebhook`.
    pub async fn set_webhook(
        &self,
        url: impl Into<String>,
        secret_token: impl Into<String>,
    ) -> Result<WebhookSet> {
        self.send(SetWebhook::new(url, secret_token)).await
    }

    /// `deleteWebhook`.
    pub async fn delete_webhook(&self) -> Result<WebhookInfo> {
        self.send(DeleteWebhook {}).await
    }

    /// `getWebhookInfo`.
    pub async fn get_webhook_info(&self) -> Result<WebhookInfo> {
        self.send(GetWebhookInfo {}).await
    }

    /// `testWebhook` (limited per day).
    pub async fn test_webhook(&self) -> Result<WebhookCheck> {
        self.send(TestWebhook {}).await
    }

    /// `sendMessage` with plain text. Use [`SendMessage`] with [`Bot::send`] for formatting.
    pub async fn send_message(
        &self,
        chat_id: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<SentMessage> {
        self.send(SendMessage::new(chat_id, text)).await
    }

    /// `sendMessage` with Zalo Markdown.
    pub async fn send_markdown(
        &self,
        chat_id: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<SentMessage> {
        self.send(SendMessage::new(chat_id, text).parse_mode(ParseMode::Markdown))
            .await
    }

    /// `sendPhoto`.
    pub async fn send_photo(
        &self,
        chat_id: impl Into<String>,
        photo_url: impl Into<String>,
        caption: Option<String>,
    ) -> Result<SentMessage> {
        let mut request = SendPhoto::new(chat_id, photo_url);
        request.caption = caption;
        self.send(request).await
    }

    /// `sendSticker`.
    pub async fn send_sticker(
        &self,
        chat_id: impl Into<String>,
        sticker_id: impl Into<String>,
    ) -> Result<SentMessage> {
        self.send(SendSticker::new(chat_id, sticker_id)).await
    }

    /// `sendVoice` (private chats only).
    pub async fn send_voice(
        &self,
        chat_id: impl Into<String>,
        voice_url: impl Into<String>,
    ) -> Result<SentMessage> {
        self.send(SendVoice::new(chat_id, voice_url)).await
    }

    /// `sendChatAction`.
    pub async fn send_chat_action(
        &self,
        chat_id: impl Into<String>,
        action: ChatAction,
    ) -> Result<()> {
        self.send(SendChatAction::new(chat_id, action)).await
    }
}

/// `{ok, result, description, error_code}`. `testWebhook` spells the code `errorCode`.
#[derive(serde::Deserialize)]
struct Envelope {
    #[serde(default)]
    ok: bool,
    #[serde(default)]
    result: Value,
    #[serde(default)]
    description: Option<String>,
    #[serde(default, alias = "errorCode")]
    error_code: Option<i64>,
}
