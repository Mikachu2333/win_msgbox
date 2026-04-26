use crate::msgbox::{MsgBoxType, MsgBtnType, raw_msgbox};
use std::slice;

/// Converts a null-terminated UTF-16 wide string pointer to a Rust `String`.
///
/// # Parameters
///
/// - `ptr`: Pointer to a null-terminated UTF-16 string. A null pointer is
///   treated as an empty string.
///
/// # Returns
///
/// A `String` containing the decoded UTF-16 data. Invalid surrogate pairs
/// are replaced with `U+FFFD` (REPLACEMENT CHARACTER).
///
/// # Safety
///
/// The caller must ensure that `ptr` points to a valid null-terminated UTF-16
/// string, or is null. The string must not be mutated while this function
/// is reading from it.
fn wide_ptr_to_string(ptr: *const u16) -> String {
    if ptr.is_null() {
        return String::new();
    }

    unsafe {
        let mut len = 0;
        while *ptr.add(len) != 0 {
            len += 1;
        }

        String::from_utf16_lossy(slice::from_raw_parts(ptr, len))
    }
}

/// Displays a custom Windows message box from C or C++ code.
///
/// This is the FFI-friendly entry point for native callers. The message and
/// title must be passed as null-terminated UTF-16 strings. A null pointer is
/// treated as an empty string.
///
/// # Parameters
///
/// - `msg`: Pointer to a null-terminated UTF-16 message string.
/// - `title`: Pointer to a null-terminated UTF-16 title string.
/// - `msgbox_type`: The message box icon style.
/// - `msgboxbtn_type`: The message box button combination.
/// - `timeout_ms`: Auto-close timeout in milliseconds. Use `0` for no timeout.
///
/// # Returns
///
/// - Returns the Windows API button result code when the user closes the dialog
///   by clicking a button, for example `1` for OK, `2` for Cancel, `6` for Yes,
///   and `7` for No.
/// - Returns `-1` when the dialog is closed automatically by the timeout
///   mechanism.
///
/// # C/C++ Declaration
///
/// ```c
/// // Link with the generated .dll/.lib
/// typedef enum {
///     MsgBoxType_Error = 0x0010,
///     MsgBoxType_Info  = 0x0040,
///     MsgBoxType_Quest = 0x0020,
///     MsgBoxType_Warn  = 0x0030,
/// } MsgBoxType;
///
/// typedef enum {
///     MsgBtnType_Ok       = 0x0000,
///     MsgBtnType_OkCancel = 0x0001,
///     MsgBtnType_YesNo    = 0x0004,
/// } MsgBtnType;
///
/// __declspec(dllimport) int __stdcall custom_msgbox_w(
///     const wchar_t* msg,
///     const wchar_t* title,
///     unsigned int msgbox_type,
///     unsigned int msgboxbtn_type,
///     unsigned long long timeout_ms
/// );
/// ```
///
/// # Example (C++)
///
/// ```cpp
/// #include <iostream>
///
/// extern "C" int __stdcall custom_msgbox_w(
///     const wchar_t* msg, const wchar_t* title,
///     unsigned int msgbox_type, unsigned int msgboxbtn_type,
///     unsigned long long timeout_ms
/// );
///
/// int main() {
///     int result = custom_msgbox_w(
///         L"Hello from C++!",
///         L"FFI Demo",
///         0x0040, // Info
///         0x0000, // OK
///         5000    // 5-second timeout
///     );
///     std::cout << "Result: " << result << std::endl;
///     return 0;
/// }
/// ```
#[unsafe(no_mangle)]
pub extern "system" fn custom_msgbox_w(
    msg: *const u16,
    title: *const u16,
    msgbox_type: u32,
    msgboxbtn_type: u32,
    timeout_ms: u64,
) -> i32 {
    let msgbox_type = match msgbox_type {
        0x0010 => MsgBoxType::Error,
        0x0020 => MsgBoxType::Quest,
        0x0030 => MsgBoxType::Warn,
        0x0040 => MsgBoxType::Info,
        _ => return 0, // Avoid UB from invalid C enum values mapped to Rust enum
    };

    let msgboxbtn_type = match msgboxbtn_type {
        0x0000 => MsgBtnType::Ok,
        0x0001 => MsgBtnType::OkCancel,
        0x0004 => MsgBtnType::YesNo,
        _ => return 0, // Avoid UB from invalid C enum values
    };

    let msg = wide_ptr_to_string(msg);
    let title = wide_ptr_to_string(title);
    raw_msgbox(msg, title, msgbox_type, msgboxbtn_type, timeout_ms)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that `wide_ptr_to_string` handles a null pointer gracefully.
    #[test]
    fn wide_ptr_to_string_null() {
        assert_eq!(wide_ptr_to_string(std::ptr::null()), "");
    }

    /// Tests that `wide_ptr_to_string` correctly decodes a simple UTF-16 string.
    #[test]
    fn wide_ptr_to_string_valid() {
        let input: Vec<u16> = "Hello\0".encode_utf16().collect();
        assert_eq!(wide_ptr_to_string(input.as_ptr()), "Hello");
    }

    /// Tests that `wide_ptr_to_string` handles an empty string.
    #[test]
    fn wide_ptr_to_string_empty() {
        let input: Vec<u16> = "\0".encode_utf16().collect();
        assert_eq!(wide_ptr_to_string(input.as_ptr()), "");
    }

    /// Tests that `wide_ptr_to_string` handles non-BMP characters (emoji).
    #[test]
    fn wide_ptr_to_string_non_bmp() {
        // U+1F600 GRINNING FACE = surrogate pair 0xD83D 0xDE00
        let input: Vec<u16> = vec![0xD83D, 0xDE00, 0x0000];
        assert_eq!(wide_ptr_to_string(input.as_ptr()), "\u{1F600}");
    }

    /// Tests that `custom_msgbox_w` with null pointers does not crash.
    ///
    /// This verifies that the FFI entry point handles null inputs safely
    /// by treating them as empty strings. The actual message box will not
    /// be displayed because `timeout_ms` is 0 and the title will be empty,
    /// causing `raw_msgbox` to fall back to the default type name.
    #[test]
    fn custom_msgbox_w_null_pointers() {
        // Null pointers should be treated as empty strings — no crash expected.
        let result = custom_msgbox_w(
            std::ptr::null(),
            std::ptr::null(),
            MsgBoxType::Info as u32,
            MsgBtnType::Ok as u32,
            0,
        );
        // With empty title, raw_msgbox falls back to "Info" as title.
        // The dialog will appear modally, so we just verify it returns a valid i32.
        assert!(result == 1 || result == -1);
    }
}
