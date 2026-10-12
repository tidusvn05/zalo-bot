//! Data the API returns and accepts.
//!
//! Every enum has an `Other` variant (or is `#[non_exhaustive]`) so values Zalo adds later never
//! break deserialization.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

macro_rules! string_enum {
    (
        $(#[$meta:meta])*
        $name:ident { $( $(#[$vmeta:meta])* $variant:ident = $value:literal ),+ $(,)? }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        #[non_exhaustive]
        pub enum $name {
            $( $(#[$vmeta])* $variant, )+
            /// A value this version does not know.
            Other(String),
        }

        impl $name {
            /// The wire value.
            pub fn as_str(&self) -> &str {
                match self {
                    $( Self::$variant => $value, )+
                    Self::Other(value) => value,
                }
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                match value {
                    $( $value => Self::$variant, )+
                    other => Self::Other(other.to_string()),
                }
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.as_str())
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_str(self.as_str())
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let value = String::deserialize(deserializer)?;
                Ok(Self::from(value.as_str()))
            }
        }
    };
}

string_enum! {
    /// The kind of an update (`event_name`).
    EventName {
        /// `message.text.received`
        Text = "message.text.received",
        /// `message.image.received`
        Image = "message.image.received",
        /// `message.sticker.received`
        Sticker = "message.sticker.received",
        /// `message.voice.received`
        Voice = "message.voice.received",
        /// `message.unsupported.received`: content withheld by Zalo (for example a protected user).
        Unsupported = "message.unsupported.received",
    }
}

string_enum! {
    /// One-to-one or group chat.
    ChatType {
        /// A private chat with one user.
        Private = "PRIVATE",
        /// A group (beta): the bot only receives mentions and replies to its own messages.
        Group = "GROUP",
    }
}

string_enum! {
    /// How `text` is formatted in `sendMessage`.
    ParseMode {
        /// Zalo Markdown: `**bold**`, `*italic*`, `~~strike~~`, headings, lists, `{red}…{/red}`.
        /// No links, code blocks or tables.
        Markdown = "markdown",
        /// A small HTML subset (`b`, `i`, `u`, `s`, `h1`–`h6`, lists, `style` colors).
        Html = "html",
    }
}

string_enum! {
    /// What `sendChatAction` shows.
    ChatAction {
        /// "typing…"
        Typing = "typing",
        /// Uploading a photo ("coming soon" in the docs).
        UploadPhoto = "upload_photo",
    }
}

string_enum! {
    /// The result of Zalo's webhook check.
    WebhookOutcome {
        /// The endpoint answered 2xx.
        Ok = "webhook.ok",
        /// 403.
        Http403 = "webhook.http.403",
        /// 404.
        Http404 = "webhook.http.404",
        /// A 5xx answer.
        Http5xx = "webhook.http.5xx",
        /// Any other status, including redirects (not followed).
        HttpOther = "webhook.http.other",
        /// TLS failure (an invalid or private certificate chain).
        Tls = "webhook.err.tls",
        /// DNS failure.
        Dns = "webhook.err.dns",
        /// No answer in time.
        Timeout = "webhook.err.timeout",
        /// Connection refused or reset.
        Connection = "webhook.err.conn",
        /// Proxy failure.
        Proxy = "webhook.err.proxy",
        /// The URL is not allowed (private address, Zalo domain).
        Blocked = "webhook.err.blocked",
        /// Another failure.
        OtherError = "webhook.err.other",
    }
}

/// The bot itself (`getMe`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BotInfo {
    /// Bot id.
    pub id: String,
    /// Name shown in Zalo.
    #[serde(default)]
    pub account_name: String,
    /// Plan (`BASIC`, …).
    #[serde(default)]
    pub account_type: String,
    /// Whether the bot may join groups.
    #[serde(default)]
    pub can_join_groups: bool,
}

/// The sender of a message.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct User {
    /// User id (opaque string).
    pub id: String,
    /// Display name.
    #[serde(default)]
    pub display_name: String,
    /// Whether the sender is a bot.
    #[serde(default)]
    pub is_bot: bool,
}

/// Where a message was sent. Reply with [`Chat::id`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Chat {
    /// Chat id (opaque string; the user id in private chats).
    pub id: String,
    /// Private or group.
    #[serde(default = "private")]
    pub chat_type: ChatType,
}

fn private() -> ChatType {
    ChatType::Private
}

/// A received message. Which optional fields are set depends on [`Update::event_name`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Message {
    /// Message id.
    #[serde(default)]
    pub message_id: String,
    /// Sender.
    pub from: User,
    /// Chat.
    pub chat: Chat,
    /// Unix time in milliseconds.
    #[serde(default)]
    pub date: i64,
    /// Text (text messages).
    #[serde(default)]
    pub text: Option<String>,
    /// Image URL (image messages).
    #[serde(default, alias = "photo_url")]
    pub photo: Option<String>,
    /// Caption sent with an image.
    #[serde(default)]
    pub caption: Option<String>,
    /// Sticker id (sticker messages).
    #[serde(default)]
    pub sticker: Option<String>,
    /// Sticker image URL (sticker messages).
    #[serde(default)]
    pub url: Option<String>,
    /// Voice message URL.
    #[serde(default)]
    pub voice_url: Option<String>,
    /// Message type (`CHAT_MESSAGE`); undocumented.
    #[serde(default)]
    pub message_type: Option<String>,
}

impl Message {
    /// Whether this is a group message.
    pub fn is_group(&self) -> bool {
        self.chat.chat_type == ChatType::Group
    }
}

/// One event from `getUpdates` or a webhook.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Update {
    /// What happened.
    pub event_name: EventName,
    /// The message.
    pub message: Message,
}

impl Update {
    /// The typed content of the message.
    pub fn content(&self) -> Content<'_> {
        let m = &self.message;
        match self.event_name {
            EventName::Text => Content::Text(m.text.as_deref().unwrap_or_default()),
            EventName::Image => Content::Image {
                url: m.photo.as_deref().unwrap_or_default(),
                caption: m.caption.as_deref(),
            },
            EventName::Sticker => Content::Sticker {
                id: m.sticker.as_deref().unwrap_or_default(),
                url: m.url.as_deref(),
            },
            EventName::Voice => Content::Voice {
                url: m.voice_url.as_deref().unwrap_or_default(),
            },
            EventName::Unsupported => Content::Unsupported,
            EventName::Other(_) => Content::Unknown,
        }
    }

    /// The chat to reply to.
    pub fn chat_id(&self) -> &str {
        &self.message.chat.id
    }
}

/// What a message contains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Content<'a> {
    /// Text.
    Text(&'a str),
    /// An image.
    Image {
        /// Image URL.
        url: &'a str,
        /// Caption.
        caption: Option<&'a str>,
    },
    /// A sticker.
    Sticker {
        /// Sticker id.
        id: &'a str,
        /// Sticker image URL.
        url: Option<&'a str>,
    },
    /// A voice message.
    Voice {
        /// Audio URL.
        url: &'a str,
    },
    /// Content Zalo withheld.
    Unsupported,
    /// An event this version does not know.
    Unknown,
}

/// A sent message.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SentMessage {
    /// Message id.
    pub message_id: String,
    /// Unix time in milliseconds.
    #[serde(default)]
    pub date: i64,
}

/// The registered webhook.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WebhookInfo {
    /// URL; empty when none is set.
    #[serde(default)]
    pub url: String,
    /// Last change, Unix time in milliseconds.
    #[serde(default)]
    pub updated_at: i64,
}

impl WebhookInfo {
    /// Whether a webhook is set (and `getUpdates` therefore does not work).
    pub fn is_set(&self) -> bool {
        !self.url.is_empty()
    }
}

/// Zalo's check of a webhook endpoint (`setWebhook`, `testWebhook`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WebhookCheck {
    /// The endpoint answered 2xx.
    #[serde(default)]
    pub ok: bool,
    /// The URL that was checked.
    #[serde(default)]
    pub url: String,
    /// What happened.
    #[serde(default = "unknown_outcome")]
    pub outcome: WebhookOutcome,
    /// Zalo's advice.
    #[serde(default)]
    pub hint: String,
}

fn unknown_outcome() -> WebhookOutcome {
    WebhookOutcome::Other(String::new())
}

/// The answer to `setWebhook`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub struct WebhookSet {
    /// The registered URL (saved even when the check fails).
    #[serde(default)]
    pub url: String,
    /// Unix time in milliseconds.
    #[serde(default)]
    pub updated_at: i64,
    /// Zalo's immediate check of the endpoint.
    #[serde(default)]
    pub verification: Option<WebhookCheck>,
}

/// A text style applied to a range of `sendMessage` text (ignored when `parse_mode` is set).
///
/// `start` and `len` count UTF-16 code units of the raw text; see [`crate::text::utf16_len`].
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct TextStyle {
    /// First UTF-16 unit.
    pub start: usize,
    /// Length in UTF-16 units.
    pub len: usize,
    /// Styles.
    #[serde(rename = "st")]
    pub styles: Vec<Style>,
}

impl TextStyle {
    /// A style for `len` UTF-16 units from `start`.
    pub fn new(start: usize, len: usize, styles: impl IntoIterator<Item = Style>) -> Self {
        Self {
            start,
            len,
            styles: styles.into_iter().collect(),
        }
    }

    /// A style covering the first occurrence of `needle` in `text`, if any.
    pub fn on(text: &str, needle: &str, styles: impl IntoIterator<Item = Style>) -> Option<Self> {
        let at = text.find(needle)?;
        Some(Self::new(
            crate::text::utf16_len(&text[..at]),
            crate::text::utf16_len(needle),
            styles,
        ))
    }
}

string_enum! {
    /// A `text_styles` style code.
    Style {
        /// Bold.
        Bold = "b",
        /// Italic.
        Italic = "i",
        /// Underline.
        Underline = "u",
        /// Strikethrough.
        Strike = "s",
        /// Font size 13.
        Size13 = "f_13",
        /// Font size 15.
        Size15 = "f_15",
        /// Font size 18.
        Size18 = "f_18",
        /// Font size 20.
        Size20 = "f_20",
        /// Default text color (`#050a19`).
        ColorDefault = "c_050a19",
        /// Green (`#15a85f`).
        Green = "c_15a85f",
        /// Yellow (`#f7b503`).
        Yellow = "c_f7b503",
        /// Orange (`#f27806`).
        Orange = "c_f27806",
        /// Red (`#db342e`).
        Red = "c_db342e",
        /// Unordered list.
        UnorderedList = "lst_1",
        /// Ordered list.
        OrderedList = "lst_2",
        /// Indent level 1.
        Indent1 = "ind_1",
        /// Indent level 2.
        Indent2 = "ind_2",
        /// Indent level 3.
        Indent3 = "ind_3",
        /// Indent level 4.
        Indent4 = "ind_4",
        /// Indent level 5.
        Indent5 = "ind_5",
    }
}
