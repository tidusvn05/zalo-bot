//! The bot token.

use std::fmt;

/// A Zalo bot token (`12345689:abc-xyz`), from the Zalo Bot Creator mini app.
///
/// The token is part of every request URL, so it is never printed: `Debug` and `Display` show
/// only the bot id before the `:`.
#[derive(Clone, PartialEq, Eq)]
pub struct Token(String);

impl Token {
    /// Wraps a token.
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into().trim().to_string())
    }

    /// Reads the token from an environment variable.
    pub fn from_env(name: &str) -> Result<Self, std::env::VarError> {
        std::env::var(name).map(Self::new)
    }

    /// The bot id: the part before `:`, safe to log.
    pub fn bot_id(&self) -> &str {
        self.0.split_once(':').map_or("", |(id, _)| id)
    }

    /// The secret value. Do not log it.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Token({}:***)", self.bot_id())
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:***", self.bot_id())
    }
}

impl From<&str> for Token {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for Token {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}
