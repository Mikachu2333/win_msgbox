use std::{
    ptr,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::{
    util::{PROCESS_NAME, normalize_text, to_wide},
    win32::{
        FindWindowW, MB_SETFOREGROUND, MB_SYSTEMMODAL, MessageBoxExW, PostMessageW, UINT, WM_CLOSE,
    },
};

/// Button combinations for Windows message boxes.
///
/// Maps to Windows API button style flags:
/// - `Ok`:       MB_OK (0x0000) — only the OK button
/// - `OkCancel`: MB_OKCANCEL (0x0001) — OK and Cancel buttons
/// - `YesNo`:    MB_YESNO (0x0004) — Yes and No buttons
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MsgBtnType {
    /// Only the OK button (MB_OK)
    Ok,
    /// OK and Cancel buttons (MB_OKCANCEL)
    OkCancel,
    /// Yes and No buttons (MB_YESNO)
    YesNo,
}

impl MsgBtnType {
    /// Converts the button type to its corresponding Windows API flag value.
    ///
    /// # Returns
    ///
    /// A `UINT` bitmask representing the button combination style.
    fn to_u32(self) -> UINT {
        match self {
            MsgBtnType::Ok => 0x0000,
            MsgBtnType::OkCancel => 0x0001,
            MsgBtnType::YesNo => 0x0004,
        }
    }
}

/// Icon styles for Windows message boxes.
///
/// Maps to Windows API icon style flags:
/// - `Error`: MB_ICONERROR (0x0010) — red X error icon
/// - `Info`:  MB_ICONINFORMATION (0x0040) — blue "i" information icon
/// - `Quest`: MB_ICONQUESTION (0x0020) — blue "?" question icon
/// - `Warn`:  MB_ICONWARNING (0x0030) — yellow "!" warning icon
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MsgBoxType {
    /// Error icon — red X (MB_ICONERROR)
    Error,
    /// Information icon — blue "i" (MB_ICONINFORMATION)
    Info,
    /// Question icon — blue "?" (MB_ICONQUESTION)
    Quest,
    /// Warning icon — yellow "!" (MB_ICONWARNING)
    Warn,
}

impl MsgBoxType {
    /// Converts the message box type to its corresponding Windows API flag value.
    ///
    /// # Returns
    ///
    /// A `UINT` bitmask representing the icon style.
    fn to_u32(self) -> UINT {
        match self {
            MsgBoxType::Error => 0x0010,
            MsgBoxType::Quest => 0x0020,
            MsgBoxType::Warn => 0x0030,
            MsgBoxType::Info => 0x0040,
        }
    }
}

impl std::fmt::Display for MsgBoxType {
    /// Formats the message box type as a human-readable title string.
    ///
    /// Used as the default dialog title when no custom title is provided.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            MsgBoxType::Error => "Error",
            MsgBoxType::Quest => "Question",
            MsgBoxType::Warn => "Warning",
            MsgBoxType::Info => "Info",
        };
        write!(f, "{}", s)
    }
}

/// Spawns a background thread that closes the message box after a timeout.
///
/// The thread sleeps for `timeout_ms` milliseconds, then finds the message box
/// window by its title and posts a `WM_CLOSE` message to it. The `timed_out`
/// flag is set to `true` so the caller can distinguish timeout from user action.
///
/// # Parameters
///
/// - `title`: The UTF-16 window title used to locate the message box window.
/// - `timeout_ms`: Timeout duration in milliseconds. If `0`, no thread is spawned.
/// - `timed_out`: Shared atomic flag set to `true` when the timeout triggers.
///
/// # Safety
///
/// This function calls `FindWindowW` and `PostMessageW` which are unsafe FFI calls.
/// The window title must uniquely identify the target message box.
fn spawn_timeout_closer(title: Vec<u16>, timeout_ms: u64, timed_out: Arc<AtomicBool>) {
    if timeout_ms == 0 {
        return;
    }

    thread::spawn(move || {
        thread::sleep(Duration::from_millis(timeout_ms));
        unsafe {
            let hwnd = FindWindowW(ptr::null(), title.as_ptr());
            if hwnd != 0 {
                timed_out.store(true, Ordering::SeqCst);
                PostMessageW(hwnd, WM_CLOSE, 0, 0);
            }
        }
    });
}

/// Core message box implementation.
///
/// Creates and displays a Windows message box with the specified icon, buttons,
/// and optional auto-close timeout. The dialog title is automatically appended
/// with the process name and a unique timestamp to ensure window uniqueness
/// for the timeout closer mechanism.
///
/// # Parameters
///
/// - `msg`: The message text to display in the dialog body.
/// - `title`: The dialog title. Falls back to the message box type name (e.g.,
///   "Error", "Info") when the provided string is empty.
/// - `msgtype`: The icon style (Error, Info, Quest, or Warn).
/// - `btntype`: The button combination (Ok, OkCancel, or YesNo).
/// - `timeout_ms`: Auto-close timeout in milliseconds. Use `0` for no timeout.
///
/// # Returns
///
/// - The Windows API button result code (e.g., `1` for OK, `2` for Cancel,
///   `6` for Yes, `7` for No).
/// - Returns `-1` when the dialog was closed by the timeout mechanism.
///
/// # Note
///
/// The title is suffixed with `[process_name] timestamp_nanos` to create a
/// unique window title, which is required for the timeout closer thread to
/// correctly identify and close the target window.
pub(crate) fn raw_msgbox(
    msg: impl ToString,
    title: impl ToString,
    msgtype: MsgBoxType,
    btntype: MsgBtnType,
    timeout_ms: u64,
) -> i32 {
    let msg = normalize_text(msg);
    let title = {
        let t = normalize_text(title);
        let original = if t.is_empty() { msgtype.to_string() } else { t };
        format!(
            "{} [{}] {}",
            original,
            PROCESS_NAME,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )
    };

    let text_w = to_wide(&msg);
    let title_w = to_wide(&title);

    let timed_out = Arc::new(AtomicBool::new(false));
    spawn_timeout_closer(title_w.clone(), timeout_ms, timed_out.clone());

    let flags = btntype.to_u32() | msgtype.to_u32() | MB_SETFOREGROUND | MB_SYSTEMMODAL;
    let result = unsafe { MessageBoxExW(0, text_w.as_ptr(), title_w.as_ptr(), flags, 0) };

    if timed_out.load(Ordering::SeqCst) {
        -1
    } else {
        result
    }
}

/// Displays a custom Windows message box.
///
/// This is the generic entry point that allows you to choose both icon style
/// and button combination at runtime.
///
/// # Parameters
///
/// - `msg`: The message text to display.
/// - `title`: The dialog title. Defaults to the selected message box type name
///   when empty.
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
pub fn custom_msgbox(
    msg: impl ToString,
    title: impl ToString,
    msgbox_type: MsgBoxType,
    msgboxbtn_type: MsgBtnType,
    timeout_ms: u64,
) -> i32 {
    raw_msgbox(msg, title, msgbox_type, msgboxbtn_type, timeout_ms)
}

/// Displays an information message box.
///
/// Shows a dialog with a blue information icon ("i") and a single OK button.
/// Suitable for informational messages and feedback to the user.
///
/// # Parameters
///
/// - `msg`: The message text to display.
/// - `title`: The dialog title. Defaults to "Info" when empty.
/// - `timeout_ms`: Auto-close timeout in milliseconds. Use `0` for no timeout.
///
/// # Returns
///
/// - `1` (IDOK) when the user clicks OK.
/// - `-1` when the dialog is closed by timeout.
#[allow(dead_code)]
pub fn info_msgbox(msg: impl ToString, title: impl ToString, timeout_ms: u64) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Info, MsgBtnType::Ok, timeout_ms)
}

/// Displays an error message box.
///
/// Shows a dialog with a red error icon (X) and a single OK button.
/// Suitable for displaying error messages and exception information.
///
/// # Parameters
///
/// - `msg`: The error text to display.
/// - `title`: The dialog title. Defaults to "Error" when empty.
/// - `timeout_ms`: Auto-close timeout in milliseconds. Use `0` for no timeout.
///
/// # Returns
///
/// - `1` (IDOK) when the user clicks OK.
/// - `-1` when the dialog is closed by timeout.
#[allow(dead_code)]
pub fn error_msgbox(msg: impl ToString, title: impl ToString, timeout_ms: u64) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Error, MsgBtnType::Ok, timeout_ms)
}

/// Displays a warning message box.
///
/// Shows a dialog with a yellow warning icon ("!") and a single OK button.
/// Suitable for cautions, warnings, and important notices.
///
/// # Parameters
///
/// - `msg`: The warning text to display.
/// - `title`: The dialog title. Defaults to "Warning" when empty.
/// - `timeout_ms`: Auto-close timeout in milliseconds. Use `0` for no timeout.
///
/// # Returns
///
/// - `1` (IDOK) when the user clicks OK.
/// - `-1` when the dialog is closed by timeout.
#[allow(dead_code)]
pub fn warn_msgbox(msg: impl ToString, title: impl ToString, timeout_ms: u64) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Warn, MsgBtnType::Ok, timeout_ms)
}

/// Displays a Yes/No question dialog.
///
/// Shows a dialog with a blue question icon ("?"), and Yes/No buttons.
/// Suitable for binary confirmation prompts.
///
/// # Parameters
///
/// - `msg`: The question text to display.
/// - `title`: The dialog title. Defaults to "Question" when empty.
/// - `timeout_ms`: Auto-close timeout in milliseconds. Use `0` for no timeout.
///
/// # Returns
///
/// - `6` (IDYES) when the user clicks Yes.
/// - `7` (IDNO) when the user clicks No.
/// - `-1` when the dialog is closed by timeout.
#[allow(dead_code)]
pub fn quest_msgbox_yesno(msg: impl ToString, title: impl ToString, timeout_ms: u64) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Quest, MsgBtnType::YesNo, timeout_ms)
}

/// Displays an OK/Cancel question dialog.
///
/// Shows a dialog with a blue question icon ("?"), and OK/Cancel buttons.
/// Suitable for operation confirmation prompts.
///
/// # Parameters
///
/// - `msg`: The question text to display.
/// - `title`: The dialog title. Defaults to "Question" when empty.
/// - `timeout_ms`: Auto-close timeout in milliseconds. Use `0` for no timeout.
///
/// # Returns
///
/// - `1` (IDOK) when the user clicks OK.
/// - `2` (IDCANCEL) when the user clicks Cancel.
/// - `-1` when the dialog is closed by timeout.
#[allow(dead_code)]
pub fn quest_msgbox_okcancel(msg: impl ToString, title: impl ToString, timeout_ms: u64) -> i32 {
    raw_msgbox(
        msg,
        title,
        MsgBoxType::Quest,
        MsgBtnType::OkCancel,
        timeout_ms,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── MsgBtnType tests ──────────────────────────────────────────────

    /// Tests that `MsgBtnType::Ok` maps to the correct Windows API flag.
    #[test]
    fn msg_btn_type_ok_flag() {
        assert_eq!(MsgBtnType::Ok.to_u32(), 0x0000);
    }

    /// Tests that `MsgBtnType::OkCancel` maps to the correct Windows API flag.
    #[test]
    fn msg_btn_type_ok_cancel_flag() {
        assert_eq!(MsgBtnType::OkCancel.to_u32(), 0x0001);
    }

    /// Tests that `MsgBtnType::YesNo` maps to the correct Windows API flag.
    #[test]
    fn msg_btn_type_yes_no_flag() {
        assert_eq!(MsgBtnType::YesNo.to_u32(), 0x0004);
    }

    // ── MsgBoxType tests ──────────────────────────────────────────────

    /// Tests that `MsgBoxType::Error` maps to the correct Windows API flag.
    #[test]
    fn msg_box_type_error_flag() {
        assert_eq!(MsgBoxType::Error.to_u32(), 0x0010);
    }

    /// Tests that `MsgBoxType::Info` maps to the correct Windows API flag.
    #[test]
    fn msg_box_type_info_flag() {
        assert_eq!(MsgBoxType::Info.to_u32(), 0x0040);
    }

    /// Tests that `MsgBoxType::Quest` maps to the correct Windows API flag.
    #[test]
    fn msg_box_type_quest_flag() {
        assert_eq!(MsgBoxType::Quest.to_u32(), 0x0020);
    }

    /// Tests that `MsgBoxType::Warn` maps to the correct Windows API flag.
    #[test]
    fn msg_box_type_warn_flag() {
        assert_eq!(MsgBoxType::Warn.to_u32(), 0x0030);
    }

    /// Tests that `MsgBoxType::Display` produces the correct default title strings.
    #[test]
    fn msg_box_type_display() {
        assert_eq!(MsgBoxType::Error.to_string(), "Error");
        assert_eq!(MsgBoxType::Info.to_string(), "Info");
        assert_eq!(MsgBoxType::Quest.to_string(), "Question");
        assert_eq!(MsgBoxType::Warn.to_string(), "Warning");
    }

    // ── normalize_text integration tests ──────────────────────────────

    /// Tests that `raw_msgbox` normalizes Windows-style line endings.
    #[test]
    fn raw_msgbox_normalizes_crlf() {
        let msg = normalize_text("line1\r\nline2");
        assert_eq!(msg, "line1\nline2");
    }

    /// Tests that `raw_msgbox` normalizes old Mac-style line endings.
    #[test]
    fn raw_msgbox_normalizes_cr() {
        let msg = normalize_text("line1\rline2");
        assert_eq!(msg, "line1\nline2");
    }

    /// Tests that `raw_msgbox` trims leading and trailing whitespace.
    #[test]
    fn raw_msgbox_trims_whitespace() {
        let msg = normalize_text("  hello world  ");
        assert_eq!(msg, "hello world");
    }

    // ── spawn_timeout_closer tests ────────────────────────────────────

    /// Tests that `spawn_timeout_closer` does nothing when timeout is 0.
    #[test]
    fn spawn_timeout_closer_zero_timeout() {
        let timed_out = Arc::new(AtomicBool::new(false));
        let title = to_wide("test");
        spawn_timeout_closer(title, 0, timed_out.clone());
        // Should return immediately without spawning a thread.
        assert!(!timed_out.load(Ordering::SeqCst));
    }

    /// Tests that `spawn_timeout_closer` does not set the timed_out flag when
    /// the target window does not exist (FindWindowW returns 0).
    ///
    /// The flag is only set when FindWindowW successfully finds the window,
    /// so with a non-existent title the flag should remain false.
    #[test]
    fn spawn_timeout_closer_window_not_found() {
        let timed_out = Arc::new(AtomicBool::new(false));
        let title = to_wide("NonExistentWindowTitle_12345");
        spawn_timeout_closer(title, 10, timed_out.clone());
        thread::sleep(Duration::from_millis(50));
        // FindWindowW returns 0 for non-existent windows, so timed_out stays false.
        assert!(!timed_out.load(Ordering::SeqCst));
    }

    // ── to_wide integration tests ─────────────────────────────────────

    /// Tests that `to_wide` produces a null-terminated UTF-16 vector.
    #[test]
    fn to_wide_null_terminated() {
        let wide = to_wide("test");
        assert_eq!(wide.last(), Some(&0));
    }

    /// Tests that `to_wide` correctly encodes ASCII text.
    #[test]
    fn to_wide_ascii() {
        let wide = to_wide("ABC");
        assert_eq!(wide, vec![0x0041, 0x0042, 0x0043, 0x0000]);
    }

    /// Tests that `to_wide` correctly encodes Unicode text.
    #[test]
    fn to_wide_unicode() {
        let wide = to_wide("你好");
        assert_eq!(wide, vec![0x4F60, 0x597D, 0x0000]);
    }
}
