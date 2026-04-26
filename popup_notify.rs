use std::{
    ptr,
    sync::Mutex,
    thread::{self, JoinHandle},
};

use crate::{
    util::to_wide,
    win32::{
        CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, CreateFontW, CreateWindowExW, DEFAULT_CHARSET,
        DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, DefWindowProcW, DeleteObject, DestroyWindow,
        DispatchMessageW, ES_AUTOVSCROLL, ES_MULTILINE, ES_READONLY, FF_SWISS, FW_NORMAL,
        GWLP_USERDATA, GetClientRect, GetDpiForWindow, GetMessageW, GetModuleHandleW,
        GetWindowLongPtrW, KillTimer, LPARAM, MSG, MoveWindow, OUT_DEFAULT_PRECIS, PostQuitMessage,
        RECT, RegisterClassExW, SPI_GETWORKAREA, SW_SHOWNA, SendMessageW,
        SetThreadDpiAwarenessContext, SetTimer, SetWindowLongPtrW, SetWindowTextW, ShowWindow,
        SystemParametersInfoW, TranslateMessage, UnregisterClassW, VARIABLE_PITCH, WM_CLOSE,
        WM_CREATE, WM_DESTROY, WM_NCDESTROY, WM_SETFONT, WM_TIMER, WNDCLASSEXW, WPARAM, WS_CAPTION,
        WS_CHILD, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WS_SYSMENU,
        WS_VISIBLE, WS_VSCROLL,
    },
};

/// Global list of notification window thread handles.
///
/// Stores join handles for all active notification window threads so they can
/// be properly joined and cleaned up when `wait_notifications()` is called.
static NOTIFY_THREADS: Mutex<Vec<JoinHandle<()>>> = Mutex::new(Vec::new());

/// Timer ID used for the auto-close countdown timer.
const TIMER_ID_AUTOCLOSE: usize = 1;

/// Default notification window width at 96 DPI (in pixels).
const NOTIFY_WIDTH_96DPI: i32 = 364;

/// Default notification window height at 96 DPI (in pixels).
const NOTIFY_HEIGHT_96DPI: i32 = 109;

/// Per-window data stored in the window's user data (GWLP_USERDATA).
///
/// This structure is created during `WM_CREATE` and cleaned up during
/// `WM_NCDESTROY`. It holds references to the edit control, font handle,
/// and state needed for the auto-close countdown display.
struct NotifyWindowData {
    /// Handle to the child edit control displaying the message text.
    edit_hwnd: crate::HWND,
    /// Handle to the created font object (to be deleted on cleanup).
    font: isize,
    /// The original window title (before countdown suffix is appended).
    original_title: String,
    /// Timestamp when the window was created (for countdown calculation).
    start_time: std::time::Instant,
    /// Auto-close timeout duration in milliseconds.
    timeout_ms: u64,
    /// Last displayed remaining seconds (to avoid redundant title updates).
    last_secs: u64,
}

/// Window procedure for the standalone notification window.
///
/// Handles the following messages:
/// - `WM_CREATE`: Initializes the edit control, creates a DPI-scaled font,
///   and allocates per-window data.
/// - `WM_TIMER`: Updates the countdown display in the title bar and closes
///   the window when the timeout expires.
/// - `WM_CLOSE`: Destroys the window.
/// - `WM_DESTROY`: Posts `WM_QUIT` to exit the message loop.
/// - `WM_NCDESTROY`: Cleans up allocated resources (font, window data).
///
/// # Parameters
///
/// - `hwnd`: Handle to the notification window.
/// - `msg`: The window message identifier.
/// - `wparam`: Message-specific parameter.
/// - `lparam`: Message-specific parameter.
///
/// # Returns
///
/// The result of message processing. Returns `0` for handled messages,
/// or the result of `DefWindowProcW` for unhandled messages.
unsafe extern "system" fn notify_wnd_proc(
    hwnd: crate::HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> isize {
    unsafe {
        match msg {
            WM_CREATE => {
                let dpi = GetDpiForWindow(hwnd);
                let dpi = if dpi == 0 { 96 } else { dpi };
                let scale = dpi as f32 / 96.0;

                let font_height = -(12.0 * scale) as i32;
                let font_name = to_wide("Microsoft YaHei UI");
                let font = CreateFontW(
                    font_height,
                    0,
                    0,
                    0,
                    FW_NORMAL,
                    0,
                    0,
                    0,
                    DEFAULT_CHARSET,
                    OUT_DEFAULT_PRECIS,
                    CLIP_DEFAULT_PRECIS,
                    CLEARTYPE_QUALITY,
                    VARIABLE_PITCH | FF_SWISS,
                    font_name.as_ptr(),
                );

                let edit_class = to_wide("EDIT");
                let edit_hwnd = CreateWindowExW(
                    0,
                    edit_class.as_ptr(),
                    ptr::null(),
                    WS_CHILD
                        | WS_VISIBLE
                        | WS_VSCROLL
                        | ES_MULTILINE
                        | ES_READONLY
                        | ES_AUTOVSCROLL,
                    0,
                    0,
                    0,
                    0,
                    hwnd,
                    0,
                    0,
                    ptr::null_mut(),
                );

                if edit_hwnd != 0 {
                    SendMessageW(edit_hwnd, WM_SETFONT, font as WPARAM, 1);

                    let mut rect: RECT = std::mem::zeroed();
                    GetClientRect(hwnd, &mut rect);
                    MoveWindow(
                        edit_hwnd,
                        0,
                        0,
                        rect.right - rect.left,
                        rect.bottom - rect.top,
                        1,
                    );
                }

                let data = Box::new(NotifyWindowData {
                    edit_hwnd,
                    font,
                    original_title: String::new(),
                    start_time: std::time::Instant::now(),
                    timeout_ms: 0,
                    last_secs: 0,
                });
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, Box::into_raw(data) as isize);

                0
            }
            WM_TIMER => {
                if wparam == TIMER_ID_AUTOCLOSE {
                    let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut NotifyWindowData;
                    if !data_ptr.is_null() {
                        let data = &mut *data_ptr;
                        let elapsed = data.start_time.elapsed().as_millis() as u64;
                        if elapsed >= data.timeout_ms {
                            KillTimer(hwnd, TIMER_ID_AUTOCLOSE);
                            DestroyWindow(hwnd);
                        } else {
                            let remaining_ms = data.timeout_ms - elapsed;
                            let secs = remaining_ms.div_ceil(1000);
                            if secs != data.last_secs {
                                data.last_secs = secs;
                                let t = format!("{} - {}", secs, data.original_title);
                                SetWindowTextW(hwnd, to_wide(&t).as_ptr());
                            }
                        }
                    } else {
                        KillTimer(hwnd, TIMER_ID_AUTOCLOSE);
                        DestroyWindow(hwnd);
                    }
                }
                0
            }
            WM_CLOSE => {
                DestroyWindow(hwnd);
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            WM_NCDESTROY => {
                let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut NotifyWindowData;
                if !data_ptr.is_null() {
                    let data = Box::from_raw(data_ptr);
                    if data.font != 0 {
                        DeleteObject(data.font);
                    }
                }
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

/// Displays a standalone notification window in the bottom-right corner of the screen.
///
/// This function creates a custom popup window that appears above the taskbar,
/// displaying the specified title and message. The window automatically closes
/// after the specified timeout, with a live countdown shown in the title bar.
///
/// # Parameters
///
/// - `title`: The window title text displayed in the title bar.
/// - `msg`: The notification message text, displayed in a scrollable read-only
///   text box with word wrap support.
/// - `timeout_ms`: Time in milliseconds before the window automatically closes.
///   Use `0` for no auto-close (window must be closed manually).
///
/// # Returns
///
/// - `true` if the notification thread was successfully spawned.
/// - `false` is never returned in the current implementation (always succeeds).
///
/// # Features
///
/// - Window appears in the bottom-right corner, above the taskbar.
/// - Always on top (`WS_EX_TOPMOST`) but does not steal focus (`WS_EX_NOACTIVATE`).
/// - Automatically scales with system DPI (per-monitor DPI aware).
/// - Text box supports word wrap and vertical scrolling for long messages.
/// - Only has a close button (no minimize/maximize).
/// - Live countdown displayed in the title bar when timeout is active.
///
/// # Thread Safety
///
/// Each notification runs in its own thread. The thread handle is stored in
/// a global `Mutex<Vec<JoinHandle>>` for later cleanup via `wait_notifications()`.
#[allow(dead_code)]
pub fn notify_msgbox_standalone(title: impl ToString, msg: impl ToString, timeout_ms: u64) -> bool {
    let title_str = title.to_string();
    let msg_str = msg.to_string();

    let handle = thread::spawn(move || {
        unsafe {
            SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let class_name_str = format!("NotifyWnd_{}", timestamp);
            let class_name = to_wide(&class_name_str);

            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: 0,
                lpfnWndProc: notify_wnd_proc,
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: GetModuleHandleW(ptr::null()),
                hIcon: 0,
                hCursor: 0,
                hbrBackground: 16, // COLOR_WINDOW + 1
                lpszMenuName: ptr::null(),
                lpszClassName: class_name.as_ptr(),
                hIconSm: 0,
            };

            if RegisterClassExW(&wc) == 0 {
                return;
            }

            let mut work_area: RECT = std::mem::zeroed();
            SystemParametersInfoW(
                SPI_GETWORKAREA,
                0,
                &mut work_area as *mut RECT as *mut std::ffi::c_void,
                0,
            );

            let static_class = to_wide("STATIC");
            let temp_hwnd = CreateWindowExW(
                0,
                static_class.as_ptr(),
                ptr::null(),
                0,
                0,
                0,
                1,
                1,
                0,
                0,
                0,
                ptr::null_mut(),
            );

            let dpi = if temp_hwnd != 0 {
                let d = GetDpiForWindow(temp_hwnd);
                DestroyWindow(temp_hwnd);
                if d == 0 { 96 } else { d }
            } else {
                96
            };

            let scale = dpi as f32 / 96.0;
            let width = (NOTIFY_WIDTH_96DPI as f32 * scale) as i32;
            let height = (NOTIFY_HEIGHT_96DPI as f32 * scale) as i32;

            let x = work_area.right - width;
            let y = work_area.bottom - height;

            let title_w = to_wide(&title_str);
            let hwnd = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
                class_name.as_ptr(),
                title_w.as_ptr(),
                WS_POPUP | WS_CAPTION | WS_SYSMENU,
                x,
                y,
                width,
                height,
                0,
                0,
                0,
                ptr::null_mut(),
            );

            if hwnd == 0 {
                UnregisterClassW(class_name.as_ptr(), 0);
                return;
            }

            let data_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut NotifyWindowData;
            if !data_ptr.is_null() {
                let data = &mut *data_ptr;
                data.original_title = title_str.clone();
                data.timeout_ms = timeout_ms;
                data.start_time = std::time::Instant::now();
                let msg_w = to_wide(&msg_str);
                SetWindowTextW(data.edit_hwnd, msg_w.as_ptr());

                if timeout_ms > 0 {
                    let secs = timeout_ms.div_ceil(1000);
                    data.last_secs = secs;
                    let t = format!("{} - {}", secs, data.original_title);
                    SetWindowTextW(hwnd, to_wide(&t).as_ptr());
                }
            }

            if timeout_ms > 0 {
                SetTimer(hwnd, TIMER_ID_AUTOCLOSE, 100, ptr::null());
            }

            ShowWindow(hwnd, SW_SHOWNA);

            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, 0, 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            UnregisterClassW(class_name.as_ptr(), 0);
        }
    });

    if let Ok(mut threads) = NOTIFY_THREADS.lock() {
        threads.retain(|t| !t.is_finished());
        threads.push(handle);
    }

    true
}

/// Waits for all notification windows to close and cleans up their threads.
///
/// This function blocks the calling thread until every notification window
/// spawned by `notify_msgbox_standalone()` has been closed and its thread
/// has terminated. Call this before the main thread exits to ensure proper
/// cleanup of all notification resources.
///
/// # Behavior
///
/// - Drains all stored thread join handles.
/// - Joins each thread, waiting for its message loop to exit.
/// - After this function returns, no notification windows remain active.
///
/// # Example
///
/// ```ignore
/// notify_msgbox_standalone("Update", "Download complete", 5000);
/// // ... do other work ...
/// wait_notifications(); // Ensure all notifications are cleaned up
/// ```
#[allow(dead_code)]
pub fn wait_notifications() {
    if let Ok(mut threads) = NOTIFY_THREADS.lock() {
        for handle in threads.drain(..) {
            let _ = handle.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests that `notify_msgbox_standalone` returns `true` (success).
    ///
    /// This test spawns a notification with a very short timeout and waits
    /// for it to complete, verifying the function returns successfully.
    #[test]
    fn notify_msgbox_standalone_returns_true() {
        let result = notify_msgbox_standalone("Test Title", "Test Message", 100);
        assert!(result);
        wait_notifications();
    }

    /// Tests that `wait_notifications` does not block indefinitely when
    /// there are no active notifications.
    #[test]
    fn wait_notifications_empty() {
        // Should return immediately with no active threads.
        wait_notifications();
    }

    /// Tests that multiple notifications can be spawned and waited on.
    #[test]
    fn multiple_notifications() {
        let r1 = notify_msgbox_standalone("A", "Message A", 50);
        let r2 = notify_msgbox_standalone("B", "Message B", 50);
        assert!(r1);
        assert!(r2);
        wait_notifications();
    }
}
