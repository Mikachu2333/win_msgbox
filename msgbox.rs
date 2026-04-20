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

/// Button combinations for message boxes
#[allow(dead_code)]
enum MsgBtnType {
    /// Only the OK button
    Ok,
    /// OK and Cancel buttons
    OkCancel,
    /// Yes and No buttons
    YesNo,
}

impl MsgBtnType {
    fn to_u32(&self) -> UINT {
        match self {
            MsgBtnType::Ok => 0x0000,
            MsgBtnType::OkCancel => 0x0001,
            MsgBtnType::YesNo => 0x0004,
        }
    }
}

/// Icon styles for message boxes and their default titles
#[allow(dead_code)]
enum MsgBoxType {
    /// Error icon (red X)
    Error,
    /// Information icon (blue i)
    Info,
    /// Question icon (blue ?)
    Quest,
    /// Warning icon (yellow !)
    Warn,
}

impl MsgBoxType {
    fn to_u32(&self) -> UINT {
        match self {
            MsgBoxType::Error => 0x0010,
            MsgBoxType::Quest => 0x0020,
            MsgBoxType::Warn => 0x0030,
            MsgBoxType::Info => 0x0040,
        }
    }
}

impl std::fmt::Display for MsgBoxType {
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

/// Core message box implementation
///
/// ### Parameters
/// - `msg`: Message text
/// - `title`: Dialog title; falls back to message type name when empty
/// - `msgtype`: Icon style
/// - `btntype`: Button combination
/// - `timeout_ms`: Auto-close timeout in milliseconds (0 means no timeout)
///
/// ### Returns
/// - `i32`: Button result code; returns -1 when closed by timeout
fn raw_msgbox(
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

/// Show an information message box
///
/// ### Parameters
/// - `msg`: Message text
/// - `title`: Dialog title; defaults to "Information"
/// - `timeout_ms`: Auto-close timeout in milliseconds
///
/// ### Behavior
/// - Blue information icon
/// - OK button only
/// - For informational feedback
#[allow(dead_code)]
pub fn info_msgbox(msg: impl ToString, title: impl ToString, timeout_ms: u64) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Info, MsgBtnType::Ok, timeout_ms)
}

/// Show an error message box
///
/// ### Parameters
/// - `msg`: Error text
/// - `title`: Dialog title; defaults to "Error"
/// - `timeout_ms`: Auto-close timeout in milliseconds
///
/// ### Behavior
/// - Red error icon
/// - OK button only
/// - For error/exception display
#[allow(dead_code)]
pub fn error_msgbox(msg: impl ToString, title: impl ToString, timeout_ms: u64) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Error, MsgBtnType::Ok, timeout_ms)
}

/// Show a warning message box
///
/// ### Parameters
/// - `msg`: Warning text
/// - `title`: Dialog title; defaults to "Warning"
/// - `timeout_ms`: Auto-close timeout in milliseconds
///
/// ### Returns
/// - `i32`: Button result (usually OK)
///
/// ### Behavior
/// - Yellow warning icon
/// - OK button only
/// - For cautions and notices
#[allow(dead_code)]
pub fn warn_msgbox(msg: impl ToString, title: impl ToString, timeout_ms: u64) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Warn, MsgBtnType::Ok, timeout_ms)
}

/// Show a Yes/No question dialog
///
/// ### Parameters
/// - `msg`: Question text
/// - `title`: Dialog title; defaults to "Question"
/// - `timeout_ms`: Auto-close timeout in milliseconds
///
/// ### Returns
/// - `i32`: Button code
///   - 6: Yes
///   - 7: No
///
/// ### Behavior
/// - Blue question icon
/// - Yes and No buttons
/// - For binary confirmations
#[allow(dead_code)]
pub fn quest_msgbox_yesno(msg: impl ToString, title: impl ToString, timeout_ms: u64) -> i32 {
    raw_msgbox(msg, title, MsgBoxType::Quest, MsgBtnType::YesNo, timeout_ms)
}

/// Show an OK/Cancel question dialog
///
/// ### Parameters
/// - `msg`: Question text
/// - `title`: Dialog title; defaults to "Question"
/// - `timeout_ms`: Auto-close timeout in milliseconds
///
/// ### Returns
/// - `i32`: Button code
///   - 1: OK
///   - 2: Cancel
///
/// ### Behavior
/// - Blue question icon
/// - OK and Cancel buttons
/// - For operation confirmations
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
