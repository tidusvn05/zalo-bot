//! Errors.

/// Everything that can go wrong in this crate.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// The API answered `ok: false`.
    #[error("Zalo API error {code}: {description}")]
    Api {
        /// `error_code` (`errorCode` on some methods).
        code: i64,
        /// `description`.
        description: String,
    },
    /// A request was rejected before sending (for example a text longer than 2000 characters).
    #[error("invalid request: {0}")]
    Invalid(String),
    /// The HTTP request failed (connection, TLS, timeout). The URL, which holds the token, is
    /// removed.
    #[error("HTTP request failed: {0}")]
    Http(#[source] reqwest::Error),
    /// The answer could not be decoded.
    #[error("unexpected answer to {method}: {message}")]
    Decode {
        /// The API method.
        method: &'static str,
        /// What did not match.
        message: String,
    },
}

impl Error {
    /// The API error code, when the API answered.
    pub fn code(&self) -> Option<i64> {
        match self {
            Self::Api { code, .. } => Some(*code),
            _ => None,
        }
    }

    /// The token is invalid or was reset (401).
    pub fn is_unauthorized(&self) -> bool {
        self.code() == Some(401)
    }

    /// Over the quota (429, or 426 for `testWebhook`'s daily limit).
    pub fn is_rate_limited(&self) -> bool {
        matches!(self.code(), Some(426 | 429))
    }

    /// `getUpdates` while a webhook is set (409).
    pub fn is_webhook_conflict(&self) -> bool {
        self.code() == Some(409)
    }

    /// Whether the same request may succeed later: network failures, timeouts, quota, 5xx.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Http(_) => true,
            Self::Api { code, .. } => matches!(code, 408 | 429) || *code >= 500,
            _ => false,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error.without_url())
    }
}

/// `Result` with this crate's [`Error`].
pub type Result<T, E = Error> = std::result::Result<T, E>;
