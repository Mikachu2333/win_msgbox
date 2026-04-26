use crate::{
    util::{PROCESS_NAME, to_wide},
    win32::{NIF_INFO, NIIF_INFO, NIM_MODIFY, NOTIFYICONDATAW, Shell_NotifyIconW},
};

/// Icon types for system tray balloon notifications.
///
/// These correspond to the `dwInfoFlags` parameter in `NOTIFYICONDATAW`.
/// Currently, only `Info` is used; `Warning` and `Error` are reserved for
/// future use.
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum NotifyIconType {
    /// Standard information icon (blue "i")
    Info,
    /// Warning icon (yellow "!") — reserved for future use
    Warning,
    /// Error icon (red "X") — reserved for future use
    Error,
}

/// Displays a balloon tip notification on an existing system tray icon.
///
/// This function uses `Shell_NotifyIconW` with `NIM_MODIFY` to show a balloon
/// notification on a tray icon that has already been added to the system tray.
/// The notification title is set to the crate name (`CARGO_PKG_NAME`).
///
/// # Parameters
///
/// - `hwnd`: Handle to the window that owns the tray icon.
/// - `msg`: The notification message text. Maximum 255 characters; longer text
///   will be silently truncated.
/// - `icon_id`: The unique identifier (`uID`) of the existing tray icon on which
///   to display the balloon notification.
///
/// # Returns
///
/// - Non-zero value on success.
/// - `0` on failure (e.g., if the tray icon with the specified `icon_id` does
///   not exist or the notification could not be displayed).
///
/// # Prerequisites
///
/// The tray icon identified by `icon_id` must have been previously added using
/// `Shell_NotifyIconW` with `NIM_ADD`. If the icon does not exist, this function
/// will fail silently.
///
/// # Limitations
///
/// - The notification title is limited to 63 characters (truncated if longer).
/// - The notification body is limited to 255 characters (truncated if longer).
/// - Balloon tips may not be displayed on newer Windows versions (8+) if the
///   tray icon was not configured with `NIF_INFO` during creation.
///
/// # Example
///
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
