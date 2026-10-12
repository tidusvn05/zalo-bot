//! Text helpers.

use zalo_bot::text::{split_message, split_text, utf16_len};

#[test]
fn utf16() {
    assert_eq!(utf16_len("chào"), 4);
    assert_eq!(utf16_len("😀"), 2);
}

#[test]
fn splitting() {
    assert_eq!(split_text("", 5), Vec::<String>::new());
    assert_eq!(
        split_text("para one\n\npara two", 12),
        ["para one", "para two"]
    );
    assert_eq!(
        split_text("line one\nline two", 12),
        ["line one", "line two"]
    );
    assert_eq!(split_text("abcdefghij", 4), ["abcd", "efgh", "ij"]);
    assert_eq!(
        split_text("Tiếng Việt có dấu", 9),
        ["Tiếng", "Việt có", "dấu"]
    );
    let long = "từ ".repeat(1500);
    let parts = split_message(&long);
    assert!(parts.iter().all(|p| p.chars().count() <= 2000));
    assert_eq!(parts.join(" ").split_whitespace().count(), 1500);
}
