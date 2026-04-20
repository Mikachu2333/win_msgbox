use crate::{
    util::{PROCESS_NAME, to_wide},
    win32::{NIF_INFO, NIIF_INFO, NIM_MODIFY, NOTIFYICONDATAW, Shell_NotifyIconW},
};

/// Notification icon types for balloon tips
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum NotifyIconType {
    Info,
    Warning,
    Error,
}

/// Displays a balloon tip notification on an existing system tray icon.
///
/// This function uses `Shell_NotifyIconW` with `NIM_MODIFY` to show a balloon
/// notification on a tray icon that has already been added to the system tray.
///
/// ### Parameters
/// - `hwnd`: Handle to the window that owns the tray icon
/// - `msg`: Notification message text (max 255 characters, will be truncated if longer)
/// - `icon_id`: The unique identifier (`uID`) of the existing tray icon to display the balloon on
///
/// ### Returns
/// - Non-zero value on success
/// - `0` on failure (e.g., if the tray icon with the specified `icon_id` does not exist)
///
/// ### Prerequisites
/// The tray icon identified by `icon_id` must have been previously added using
/// `Shell_NotifyIconW` with `NIM_ADD`. If the icon does not exist, this function will fail.
///
/// ### Example
/// ```ignore
/// // Assuming a tray icon with ID 1 has been added to the system tray
/// let result = notify_msgbox(hwnd, "Operation completed successfully", 1);
/// if result == 0 {
///     eprintln!("Failed to show notification");
/// }
/// ```
#[allow(dead_code)]
pub fn notify_msgbox(hwnd: crate::HWND, msg: impl ToString, icon_id: u32) -> i32 {
    let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    nid.hWnd = hwnd;
    nid.uID = icon_id;
    nid.uFlags = NIF_INFO;
    nid.dwInfoFlags = NIIF_INFO;

    let title_w = to_wide(PROCESS_NAME);
    let msg_w = to_wide(msg);

    for (i, &c) in title_w.iter().take(63).enumerate() {
        nid.szInfoTitle[i] = c;
    }
    for (i, &c) in msg_w.iter().take(255).enumerate() {
        nid.szInfo[i] = c;
    }

    unsafe { Shell_NotifyIconW(NIM_MODIFY, &nid) }
}
