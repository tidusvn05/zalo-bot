//! One type per API method. Build one, then send it with [`Bot::send`](crate::Bot::send), or use
//! the shortcut methods on [`Bot`](crate::Bot).
//!
//! ```no_run
//! # async fn run(bot: zalo_bot::Bot) -> zalo_bot::Result<()> {
//! use zalo_bot::methods::SendMessage;
//! use zalo_bot::ParseMode;
//!
//! let sent = bot
//!     .send(SendMessage::new("chat-id", "**Xin chào**").parse_mode(ParseMode::Markdown))
//!     .await?;
//! println!("sent {}", sent.message_id);
//! # Ok(()) }
//! ```

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::{Error, Result};
use crate::types::{
    BotInfo, ChatAction, ParseMode, SentMessage, TextStyle, Update, WebhookCheck, WebhookInfo,
    WebhookSet,
};

/// Longest `text` or `caption` the API accepts, in characters.
pub const MAX_TEXT_CHARS: usize = 2000;

/// Accepted webhook secret lengths.
pub const SECRET_LEN: std::ops::RangeInclusive<usize> = 8..=256;

/// An API method: its name, JSON body and result type.
pub trait Request: Serialize {
    /// Method name in the URL (`sendMessage`).
    const METHOD: &'static str;
    /// The `result` of a successful call.
    type Response: DeserializeOwned;

    /// Checks the request before it is sent.
    fn validate(&self) -> Result<()> {
        Ok(())
    }

    /// Reads `result` (`null` when the answer had none).
    fn decode(result: serde_json::Value) -> serde_json::Result<Self::Response> {
        serde_json::from_value(result)
    }
}

fn check_text(field: &str, text: &str) -> Result<()> {
    let chars = text.chars().count();
    if chars == 0 || chars > MAX_TEXT_CHARS {
        return Err(Error::Invalid(format!(
            "{field} must be 1–{MAX_TEXT_CHARS} characters, got {chars} (split it with zalo_bot::text::split_text)"
        )));
    }
    Ok(())
}

fn check_chat(chat_id: &str) -> Result<()> {
    if chat_id.is_empty() {
        return Err(Error::Invalid("chat_id is empty".into()));
    }
    Ok(())
}

fn check_url(field: &str, url: &str) -> Result<()> {
    if !url.starts_with("https://") && !url.starts_with("http://") {
        return Err(Error::Invalid(format!("{field} must be an http(s) URL")));
    }
    Ok(())
}

/// `getMe`: the bot's own account; a cheap way to check the token.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetMe {}

impl Request for GetMe {
    const METHOD: &'static str = "getMe";
    type Response = BotInfo;
}

/// `getUpdates`: waits up to `timeout` seconds for the next event.
///
/// The API returns at most one update per call, with no offset: a returned update counts as
/// consumed. Does not work while a webhook is set (error 409). Prefer [`Bot::updates`](crate::Bot::updates),
/// which loops for you.
#[derive(Debug, Clone, Serialize)]
pub struct GetUpdates {
    /// Seconds to wait (the API documents a string; numbers are accepted).
    pub timeout: u64,
}

impl Default for GetUpdates {
    fn default() -> Self {
        Self { timeout: 30 }
    }
}

impl Request for GetUpdates {
    const METHOD: &'static str = "getUpdates";
    type Response = Vec<Update>;

    /// Accepts nothing (`null`, `{}`), one update, or a list.
    fn decode(result: serde_json::Value) -> serde_json::Result<Vec<Update>> {
        use serde_json::Value;
        match result {
            Value::Null => Ok(Vec::new()),
            Value::Object(ref map) if map.is_empty() => Ok(Vec::new()),
            Value::Array(_) => serde_json::from_value(result),
            other => serde_json::from_value(other).map(|u| vec![u]),
        }
    }
}

/// `setWebhook`: deliver events to `url` (HTTPS, public, no redirects) with the
/// `X-Bot-Api-Secret-Token` header set to `secret_token`.
#[derive(Debug, Clone, Serialize)]
pub struct SetWebhook {
    /// Endpoint.
    pub url: String,
    /// Secret, 8–256 characters.
    pub secret_token: String,
}

impl SetWebhook {
    /// A webhook at `url` with `secret_token`.
    pub fn new(url: impl Into<String>, secret_token: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            secret_token: secret_token.into(),
        }
    }
}

impl Request for SetWebhook {
    const METHOD: &'static str = "setWebhook";
    type Response = WebhookSet;

    fn validate(&self) -> Result<()> {
        if !self.url.starts_with("https://") {
            return Err(Error::Invalid("the webhook URL must use https".into()));
        }
        let len = self.secret_token.chars().count();
        if !SECRET_LEN.contains(&len) {
            return Err(Error::Invalid(format!(
                "secret_token must be 8–256 characters, got {len}"
            )));
        }
        Ok(())
    }
}

/// `deleteWebhook`: stop webhook delivery (needed before `getUpdates`).
#[derive(Debug, Clone, Default, Serialize)]
pub struct DeleteWebhook {}

impl Request for DeleteWebhook {
    const METHOD: &'static str = "deleteWebhook";
    type Response = WebhookInfo;

    /// The docs show `{url, updated_at}` but also say "returns True": accept both.
    fn decode(result: serde_json::Value) -> serde_json::Result<WebhookInfo> {
        if result.is_object() {
            serde_json::from_value(result)
        } else {
            Ok(WebhookInfo {
                url: String::new(),
                updated_at: 0,
            })
        }
    }
}

/// `getWebhookInfo`: the registered webhook.
#[derive(Debug, Clone, Default, Serialize)]
pub struct GetWebhookInfo {}

impl Request for GetWebhookInfo {
    const METHOD: &'static str = "getWebhookInfo";
    type Response = WebhookInfo;
}

/// `testWebhook`: asks Zalo to call the webhook now. Limited per bot per day (error 426).
#[derive(Debug, Clone, Default, Serialize)]
pub struct TestWebhook {}

impl Request for TestWebhook {
    const METHOD: &'static str = "testWebhook";
    type Response = WebhookCheck;
}

/// `sendMessage`: a text message of 1–2000 characters.
#[derive(Debug, Clone, Serialize)]
pub struct SendMessage {
    /// Chat id.
    pub chat_id: String,
    /// Text.
    pub text: String,
    /// Formatting of `text`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parse_mode: Option<ParseMode>,
    /// Range styles (ignored by Zalo when `parse_mode` is set).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub text_styles: Vec<TextStyle>,
}

impl SendMessage {
    /// Plain `text` to `chat_id`.
    pub fn new(chat_id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            text: text.into(),
            parse_mode: None,
            text_styles: Vec::new(),
        }
    }

    /// Interprets `text` as Markdown or HTML.
    pub fn parse_mode(mut self, mode: ParseMode) -> Self {
        self.parse_mode = Some(mode);
        self
    }

    /// Adds a range style.
    pub fn style(mut self, style: TextStyle) -> Self {
        self.text_styles.push(style);
        self
    }
}

impl Request for SendMessage {
    const METHOD: &'static str = "sendMessage";
    type Response = SentMessage;

    fn validate(&self) -> Result<()> {
        check_chat(&self.chat_id)?;
        check_text("text", &self.text)
    }
}

/// `sendPhoto`: an image by URL, with an optional caption.
#[derive(Debug, Clone, Serialize)]
pub struct SendPhoto {
    /// Chat id.
    pub chat_id: String,
    /// Image URL.
    pub photo: String,
    /// Caption, 1–2000 characters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
}

impl SendPhoto {
    /// The image at `photo_url` to `chat_id`.
    pub fn new(chat_id: impl Into<String>, photo_url: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            photo: photo_url.into(),
            caption: None,
        }
    }

    /// Adds a caption.
    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = Some(caption.into());
        self
    }
}

impl Request for SendPhoto {
    const METHOD: &'static str = "sendPhoto";
    type Response = SentMessage;

    fn validate(&self) -> Result<()> {
        check_chat(&self.chat_id)?;
        check_url("photo", &self.photo)?;
        match &self.caption {
            Some(caption) => check_text("caption", caption),
            None => Ok(()),
        }
    }
}

/// `sendSticker`: a sticker by id (from <https://stickers.zaloapp.com>).
#[derive(Debug, Clone, Serialize)]
pub struct SendSticker {
    /// Chat id.
    pub chat_id: String,
    /// Sticker id.
    pub sticker: String,
}

impl SendSticker {
    /// Sticker `sticker_id` to `chat_id`.
    pub fn new(chat_id: impl Into<String>, sticker_id: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            sticker: sticker_id.into(),
        }
    }
}

impl Request for SendSticker {
    const METHOD: &'static str = "sendSticker";
    type Response = SentMessage;

    fn validate(&self) -> Result<()> {
        check_chat(&self.chat_id)?;
        if self.sticker.is_empty() {
            return Err(Error::Invalid("sticker is empty".into()));
        }
        Ok(())
    }
}

/// `sendVoice`: an `.aac` audio file by URL. Private chats only: Zalo accepts it for groups but
/// does not deliver it.
#[derive(Debug, Clone, Serialize)]
pub struct SendVoice {
    /// Chat id.
    pub chat_id: String,
    /// `.aac` URL.
    pub voice_url: String,
}

impl SendVoice {
    /// The audio at `voice_url` to `chat_id`.
    pub fn new(chat_id: impl Into<String>, voice_url: impl Into<String>) -> Self {
        Self {
            chat_id: chat_id.into(),
            voice_url: voice_url.into(),
        }
    }
}

impl Request for SendVoice {
    const METHOD: &'static str = "sendVoice";
    type Response = SentMessage;

    fn validate(&self) -> Result<()> {
        check_chat(&self.chat_id)?;
        check_url("voice_url", &self.voice_url)
    }
}

/// `sendChatAction`: shows "typing…" (or "uploading a photo") for a few seconds.
#[derive(Debug, Clone, Serialize)]
pub struct SendChatAction {
    /// Chat id.
    pub chat_id: String,
    /// Action.
    pub action: ChatAction,
}

impl SendChatAction {
    /// `action` in `chat_id`.
    pub fn new(chat_id: impl Into<String>, action: ChatAction) -> Self {
        Self {
            chat_id: chat_id.into(),
            action,
        }
    }
}

impl Request for SendChatAction {
    const METHOD: &'static str = "sendChatAction";
    type Response = ();

    /// The answer carries nothing useful (no `result`, `true` or `{}`).
    fn decode(_: serde_json::Value) -> serde_json::Result<()> {
        Ok(())
    }

    fn validate(&self) -> Result<()> {
        check_chat(&self.chat_id)
    }
}
