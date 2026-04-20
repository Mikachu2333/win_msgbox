pub(crate) static PROCESS_NAME: &str = std::env!("CARGO_PKG_NAME");

pub(crate) fn normalize_text(text: impl ToString) -> String {
    let result = text.to_string().replace("\r\n", "\n").replace('\r', "\n");
    result.trim().to_string()
}

pub(crate) fn to_wide(text: impl ToString) -> Vec<u16> {
    let text = text.to_string();
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
