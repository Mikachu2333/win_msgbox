/// The name of the current crate, obtained at compile time from `CARGO_PKG_NAME`.
///
/// Used as the default title for tray notifications and as a suffix in
/// message box titles to help identify the source process.
pub(crate) static PROCESS_NAME: &str = std::env!("CARGO_PKG_NAME");

/// Normalizes text by converting line endings and trimming whitespace.
///
/// Converts `\r\n` (Windows) and `\r` (old Mac) line endings to `\n` (Unix),
/// then trims leading and trailing whitespace.
///
/// # Parameters
///
/// - `text`: The input text to normalize.
///
/// # Returns
///
/// A `String` with normalized line endings and trimmed whitespace.
pub(crate) fn normalize_text(text: impl ToString) -> String {
    let result = text.to_string().replace("\r\n", "\n").replace('\r', "\n");
    result.trim().to_string()
}

/// Converts a string to a null-terminated UTF-16 wide character vector.
///
/// This is a helper for calling Windows API functions that require
/// `LPCWSTR` (pointer to null-terminated UTF-16 string).
///
/// # Parameters
///
/// - `text`: The input text to convert.
///
/// # Returns
///
/// A `Vec<u16>` containing the UTF-16 encoded string, terminated with a
/// null character (`0x0000`).
pub(crate) fn to_wide(text: impl ToString) -> Vec<u16> {
    let text = text.to_string();
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
