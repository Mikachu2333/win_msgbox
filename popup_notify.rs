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
        SystemParametersInfoW, TranslateMessage, UINT, UnregisterClassW, VARIABLE_PITCH, WM_CLOSE,
        WM_CREATE, WM_DESTROY, WM_NCDESTROY, WM_SETFONT, WM_TIMER, WNDCLASSEXW, WPARAM, WS_CAPTION,
        WS_CHILD, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_POPUP, WS_SYSMENU,
        WS_VISIBLE, WS_VSCROLL,
    },
};

/// Global list of notification window threads
static NOTIFY_THREADS: Mutex<Vec<JoinHandle<()>>> = Mutex::new(Vec::new());

// Timer ID for auto-close
const TIMER_ID_AUTOCLOSE: usize = 1;

// Windows notification typical size at 96 DPI
const NOTIFY_WIDTH_96DPI: i32 = 364;
const NOTIFY_HEIGHT_96DPI: i32 = 109;

/// Data passed to the notification window
struct NotifyWindowData {
    edit_hwnd: crate::HWND,
    font: isize,
    original_title: String,
    start_time: std::time::Instant,
    timeout_ms: u64,
    last_secs: u64,
}

/// Window procedure for the notification window
unsafe extern "system" fn notify_wnd_proc(
    hwnd: crate::HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> isize {
    unsafe {
        match msg {
            WM_CREATE => {
                // Get DPI for proper scaling
                let dpi = GetDpiForWindow(hwnd);
                let dpi = if dpi == 0 { 96 } else { dpi };
                let scale = dpi as f32 / 96.0;

                // Create font scaled for DPI (Microsoft YaHei UI, 12pt at 96 DPI)
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

                // Create the edit control for text display
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
                    // Set the font
                    SendMessageW(edit_hwnd, WM_SETFONT, font as WPARAM, 1);

                    // Resize the edit control to fill the client area
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

                // Store data in window user data
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
                // Clean up allocated data
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
/// after the specified timeout.
///
/// ### Parameters
/// - `title`: Window title text
/// - `msg`: Notification message text (displayed in a scrollable, read-only text box)
/// - `timeout_ms`: Time in milliseconds before the window automatically closes
///
/// ### Returns
/// - `true` on success
/// - `false` on failure
///
/// ### Features
/// - Window appears in the bottom-right corner, above the taskbar
/// - Always on top but does not steal focus
/// - Automatically scales with system DPI
/// - Text box supports word wrap and vertical scrolling for long messages
/// - Only has a close button (no minimize/maximize)
#[allow(dead_code)]
pub fn notify_msgbox_standalone(title: impl ToString, msg: impl ToString, timeout_ms: u64) -> bool {
    let title_str = title.to_string();
    let msg_str = msg.to_string();

    let handle = thread::spawn(move || {
        unsafe {
            // Enable Per-Monitor DPI awareness for this thread
            SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);

            // Generate a unique class name using timestamp
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let class_name_str = format!("NotifyWnd_{}", timestamp);
            let class_name = to_wide(&class_name_str);

            // Register window class
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

            // Get work area (screen area excluding taskbar)
            let mut work_area: RECT = std::mem::zeroed();
            SystemParametersInfoW(
                SPI_GETWORKAREA,
                0,
                &mut work_area as *mut RECT as *mut std::ffi::c_void,
                0,
            );

            // Create a temporary window to get DPI using system class
            // to avoid triggering PostQuitMessage from our custom wndproc
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

            // Scale dimensions for DPI
            let scale = dpi as f32 / 96.0;
            let width = (NOTIFY_WIDTH_96DPI as f32 * scale) as i32;
            let height = (NOTIFY_HEIGHT_96DPI as f32 * scale) as i32;

            // Position: bottom-right, above taskbar
            let x = work_area.right - width;
            let y = work_area.bottom - height;

            // Create the notification window
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

            // Set the message text in the edit control
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

            // Set auto-close timer
            if timeout_ms > 0 {
                SetTimer(hwnd, TIMER_ID_AUTOCLOSE, 100, ptr::null());
            }

            // Show window without activating
            ShowWindow(hwnd, SW_SHOWNA);

            // Message loop
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, 0, 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            // Cleanup
            UnregisterClassW(class_name.as_ptr(), 0);
        }
    });

    // Store the thread handle for later joining and cleanup finished threads
    if let Ok(mut threads) = NOTIFY_THREADS.lock() {
        threads.retain(|t| !t.is_finished());
        threads.push(handle);
    }

    true
}

/// Waits for all notification windows to close.
///
/// Call this before the main thread exits to ensure all notification
/// windows have been properly closed and cleaned up.
#[allow(dead_code)]
pub fn wait_notifications() {
    if let Ok(mut threads) = NOTIFY_THREADS.lock() {
        for handle in threads.drain(..) {
            let _ = handle.join();
        }
    }
}
