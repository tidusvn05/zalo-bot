//! Text helpers.

use crate::methods::MAX_TEXT_CHARS;

/// Length in UTF-16 code units, the unit of [`TextStyle`](crate::TextStyle) offsets.
pub fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

/// Splits `text` into chunks of at most `max_chars` characters (use
/// [`MAX_TEXT_CHARS`] for messages), preferring paragraph, line and word boundaries. Chunks are
/// trimmed and never empty.
///
/// ```
/// let parts = zalo_bot::text::split_text("one two three", 7);
/// assert_eq!(parts, ["one two", "three"]);
/// ```
pub fn split_text(text: &str, max_chars: usize) -> Vec<String> {
    let max_chars = max_chars.max(1);
    let mut parts = Vec::new();
    let mut rest = text.trim();
    while !rest.is_empty() {
        if rest.chars().count() <= max_chars {
            parts.push(rest.to_string());
            break;
        }
        // Byte index just after the `max_chars`-th character.
        let limit = rest
            .char_indices()
            .nth(max_chars)
            .map_or(rest.len(), |(i, _)| i);
        let window = &rest[..limit];
        let cut = if rest[limit..].starts_with(char::is_whitespace) {
            limit // the chunk ends exactly at a boundary
        } else {
            ["\n\n", "\n", " "]
                .iter()
                .find_map(|sep| window.rfind(sep).filter(|&i| i > 0))
                .unwrap_or(limit)
        };
        parts.push(rest[..cut].trim_end().to_string());
        rest = rest[cut..].trim_start();
    }
    parts
}

/// Splits `text` for `sendMessage` (2000 characters per message).
pub fn split_message(text: &str) -> Vec<String> {
    split_text(text, MAX_TEXT_CHARS)
}
