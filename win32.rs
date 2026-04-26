//! Windows API type aliases, constants, and FFI declarations.
//!
//! This module provides the raw Windows API bindings used by the crate,
//! including type definitions, message/window style constants, and
//! `extern "system"` function declarations from `User32`, `Gdi32`, and `Shell32`.

/// Window handle (`HWND`). A value of `0` represents no window (null).
#[allow(clippy::upper_case_acronyms)]
pub type HWND = isize;

/// Pointer to a constant null-terminated UTF-16 string (`LPCWSTR`).
#[allow(clippy::upper_case_acronyms)]
pub type LPCWSTR = *const u16;

/// 16-bit unsigned integer (`WORD`).
#[allow(clippy::upper_case_acronyms)]
pub type WORD = u16;

/// Message parameter (`WPARAM`), typically `usize` on 64-bit Windows.
#[allow(clippy::upper_case_acronyms)]
pub type WPARAM = usize;

/// Message parameter (`LPARAM`), typically `isize` on 64-bit Windows.
#[allow(clippy::upper_case_acronyms)]
pub type LPARAM = isize;

/// 32-bit boolean (`BOOL`). `0` = false, non-zero = true.
#[allow(clippy::upper_case_acronyms)]
pub type BOOL = i32;

/// Makes the message box system-modal (MB_SYSTEMMODAL).
/// The dialog is displayed on top of all windows and requires user response.
pub const MB_SYSTEMMODAL: u32 = 0x1000;

/// Brings the message box to the foreground (MB_SETFOREGROUND).
pub const MB_SETFOREGROUND: u32 = 0x10000;

/// Window message: close the window (WM_CLOSE).
pub const WM_CLOSE: u32 = 0x0010;

/// Shell_NotifyIconW message: modify an existing tray icon (NIM_MODIFY).
pub const NIM_MODIFY: u32 = 0x00000001;

/// NOTIFYICONDATAW flag: the `szInfo` and `szInfoTitle` fields are valid (NIF_INFO).
pub const NIF_INFO: u32 = 0x00000010;

/// Balloon notification info flag: show an information icon (NIIF_INFO).
pub const NIIF_INFO: u32 = 0x00000001;

/// Popup window style (WS_POPUP).
pub const WS_POPUP: u32 = 0x80000000;

/// Window has a title bar (WS_CAPTION = WS_BORDER | WS_DLGFRAME).
pub const WS_CAPTION: u32 = 0x00C00000;

/// Window has a system menu (close button) (WS_SYSMENU).
pub const WS_SYSMENU: u32 = 0x00080000;

/// Initially visible window (WS_VISIBLE).
pub const WS_VISIBLE: u32 = 0x10000000;

/// Child window style (WS_CHILD).
pub const WS_CHILD: u32 = 0x40000000;

/// Window has a vertical scroll bar (WS_VSCROLL).
pub const WS_VSCROLL: u32 = 0x00200000;

/// Window is topmost (always on top) (WS_EX_TOPMOST).
pub const WS_EX_TOPMOST: u32 = 0x00000008;

/// Window does not steal focus when shown (WS_EX_NOACTIVATE).
pub const WS_EX_NOACTIVATE: u32 = 0x08000000;

/// Tool window style (thin title bar, no taskbar entry) (WS_EX_TOOLWINDOW).
pub const WS_EX_TOOLWINDOW: u32 = 0x00000080;

/// Multi-line edit control (ES_MULTILINE).
pub const ES_MULTILINE: u32 = 0x0004;

/// Read-only edit control (ES_READONLY).
pub const ES_READONLY: u32 = 0x0800;

/// Auto-vertical scroll for edit control (ES_AUTOVSCROLL).
pub const ES_AUTOVSCROLL: u32 = 0x0040;

/// Sent when a window is created (WM_CREATE).
pub const WM_CREATE: u32 = 0x0001;

/// Sent when a window is being destroyed (WM_DESTROY).
pub const WM_DESTROY: u32 = 0x0002;

/// Sent when a timer fires (WM_TIMER).
pub const WM_TIMER: u32 = 0x0113;

/// Sets the font for a control (WM_SETFONT).
pub const WM_SETFONT: u32 = 0x0030;

/// Sent after the window is destroyed (WM_NCDESTROY).
pub const WM_NCDESTROY: u32 = 0x0082;

/// Retrieves the work area (screen excluding taskbar) (SPI_GETWORKAREA).
pub const SPI_GETWORKAREA: u32 = 0x0030;

/// Index for retrieving/setting user data associated with a window (GWLP_USERDATA).
pub const GWLP_USERDATA: i32 = -21;

/// Normal font weight (FW_NORMAL = 400).
pub const FW_NORMAL: i32 = 400;

/// Default character set (DEFAULT_CHARSET).
pub const DEFAULT_CHARSET: u32 = 1;

/// Default output precision (OUT_DEFAULT_PRECIS).
pub const OUT_DEFAULT_PRECIS: u32 = 0;

/// Default clip precision (CLIP_DEFAULT_PRECIS).
pub const CLIP_DEFAULT_PRECIS: u32 = 0;

/// ClearType antialiasing quality (CLEARTYPE_QUALITY).
pub const CLEARTYPE_QUALITY: u32 = 5;

/// Variable-pitch font family (VARIABLE_PITCH).
pub const VARIABLE_PITCH: u32 = 2;

/// Swiss (sans-serif) font family (FF_SWISS).
pub const FF_SWISS: u32 = 0x20;

/// Shows the window without activating it (SW_SHOWNA).
pub const SW_SHOWNA: i32 = 8;

/// Per-monitor DPI awareness context v2 (DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2).
pub const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: isize = -4;

/// Windows `RECT` structure — defines the coordinates of a rectangle.
#[repr(C)]
#[derive(Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub struct RECT {
    /// X-coordinate of the left edge.
    pub left: i32,
    /// Y-coordinate of the top edge.
    pub top: i32,
    /// X-coordinate of the right edge.
    pub right: i32,
    /// Y-coordinate of the bottom edge.
    pub bottom: i32,
}

/// Windows `WNDCLASSEXW` structure — window class information (wide char version).
#[repr(C)]
#[allow(non_snake_case, clippy::upper_case_acronyms)]
pub struct WNDCLASSEXW {
    /// Size of the structure, in bytes.
    pub cbSize: u32,
    /// Class style flags.
    pub style: u32,
    /// Pointer to the window procedure.
    pub lpfnWndProc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> isize,
    /// Extra bytes to allocate after the window class structure.
    pub cbClsExtra: i32,
    /// Extra bytes to allocate after the window instance.
    pub cbWndExtra: i32,
    /// Handle to the instance that contains the window procedure.
    pub hInstance: isize,
    /// Handle to the class icon.
    pub hIcon: isize,
    /// Handle to the class cursor.
    pub hCursor: isize,
    /// Handle to the class background brush.
    pub hbrBackground: isize,
    /// Pointer to a null-terminated string for the menu name.
    pub lpszMenuName: *const u16,
    /// Pointer to a null-terminated string for the class name.
    pub lpszClassName: *const u16,
    /// Handle to a small icon associated with the window class.
    pub hIconSm: isize,
}

/// Windows `MSG` structure — contains message information from a thread's message queue.
#[repr(C)]
#[allow(non_snake_case, clippy::upper_case_acronyms)]
pub struct MSG {
    /// Handle to the window whose window procedure receives the message.
    pub hwnd: HWND,
    /// The message identifier.
    pub message: u32,
    /// Additional message-specific information.
    pub wParam: WPARAM,
    /// Additional message-specific information.
    pub lParam: LPARAM,
    /// The time at which the message was posted.
    pub time: u32,
    /// X-coordinate of the cursor position (in screen coordinates).
    pub pt_x: i32,
    /// Y-coordinate of the cursor position (in screen coordinates).
    pub pt_y: i32,
}

/// Windows `NOTIFYICONDATAW` structure — contains information for the system tray icon.
#[repr(C)]
#[allow(non_snake_case, clippy::upper_case_acronyms)]
pub struct NOTIFYICONDATAW {
    /// Size of the structure, in bytes.
    pub cbSize: u32,
    /// Handle to the window that receives notification messages.
    pub hWnd: HWND,
    /// Application-defined identifier for the icon.
    pub uID: u32,
    /// Flags indicating which fields are valid (NIF_*).
    pub uFlags: u32,
    /// Application-defined callback message for the window.
    pub uCallbackMessage: u32,
    /// Handle to the icon.
    pub hIcon: isize,
    /// Tooltip text (max 128 characters).
    pub szTip: [u16; 128],
    /// State of the icon.
    pub dwState: u32,
    /// State mask.
    pub dwStateMask: u32,
    /// Balloon notification text (max 256 characters).
    pub szInfo: [u16; 256],
    /// Timeout or version information.
    pub uTimeoutOrVersion: u32,
    /// Balloon notification title (max 64 characters).
    pub szInfoTitle: [u16; 64],
    /// Icon flags for the balloon notification (NIIF_*).
    pub dwInfoFlags: u32,
    /// GUID for the icon (Windows 7+).
    pub guidItem: [u8; 16],
    /// Handle to a custom balloon icon.
    pub hBalloonIcon: isize,
}

#[link(name = "User32")]
unsafe extern "system" {
    /// Displays a modal message box (wide character version).
    ///
    /// # Parameters
    ///
    /// - `hWnd`: Handle to the owner window (0 for no owner).
    /// - `lpText`: Pointer to the null-terminated message text.
    /// - `lpCaption`: Pointer to the null-terminated dialog title.
    /// - `uType`: Style flags (button combination | icon | modal type).
    /// - `wLanguageId`: Language identifier for the dialog (0 = system default).
    ///
    /// # Returns
    ///
    /// The button ID pressed by the user (e.g., IDOK=1, IDCANCEL=2, IDYES=6, IDNO=7).
    /// Returns 0 if the function fails.
    pub fn MessageBoxExW(
        hWnd: HWND,
        lpText: LPCWSTR,
        lpCaption: LPCWSTR,
        uType: u32,
        wLanguageId: WORD,
    ) -> i32;

    /// Finds a top-level window by its class name and window title.
    ///
    /// # Parameters
    ///
    /// - `lpClassName`: Pointer to the class name (null to match any class).
    /// - `lpWindowName`: Pointer to the window title.
    ///
    /// # Returns
    ///
    /// A handle to the found window, or `0` if not found.
    pub fn FindWindowW(lpClassName: LPCWSTR, lpWindowName: LPCWSTR) -> HWND;

    /// Posts a message to a window's message queue (asynchronous).
    ///
    /// # Parameters
    ///
    /// - `hWnd`: Handle to the target window.
    /// - `Msg`: The message to post.
    /// - `wParam`: First message parameter.
    /// - `lParam`: Second message parameter.
    ///
    /// # Returns
    ///
    /// Non-zero if the message was posted; zero on failure.
    pub fn PostMessageW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> BOOL;

    /// Creates an overlapped, pop-up, or child window (wide character version).
    ///
    /// # Parameters
    ///
    /// - `dwExStyle`: Extended window style (WS_EX_*).
    /// - `lpClassName`: Pointer to the registered class name.
    /// - `lpWindowName`: Pointer to the window title.
    /// - `dwStyle`: Window style (WS_*).
    /// - `x`: Initial horizontal position.
    /// - `y`: Initial vertical position.
    /// - `nWidth`: Window width.
    /// - `nHeight`: Window height.
    /// - `hWndParent`: Handle to the parent or owner window.
    /// - `hMenu`: Handle to a menu or child-window identifier.
    /// - `hInstance`: Handle to the module instance.
    /// - `lpParam`: Pointer to creation data.
    ///
    /// # Returns
    ///
    /// A handle to the new window, or `0` on failure.
    pub fn CreateWindowExW(
        dwExStyle: u32,
        lpClassName: *const u16,
        lpWindowName: *const u16,
        dwStyle: u32,
        x: i32,
        y: i32,
        nWidth: i32,
        nHeight: i32,
        hWndParent: HWND,
        hMenu: isize,
        hInstance: isize,
        lpParam: *mut std::ffi::c_void,
    ) -> HWND;

    /// Destroys a window and sends `WM_DESTROY` and `WM_NCDESTROY` to it.
    ///
    /// # Returns
    ///
    /// Non-zero on success; zero on failure.
    pub fn DestroyWindow(hWnd: HWND) -> i32;

    /// Registers a window class for subsequent use in `CreateWindowExW`.
    ///
    /// # Returns
    ///
    /// A class atom that uniquely identifies the class, or `0` on failure.
    pub fn RegisterClassExW(lpWndClass: *const WNDCLASSEXW) -> u16;

    /// Unregisters a window class, freeing the memory used for the class.
    ///
    /// # Returns
    ///
    /// Non-zero on success; zero on failure.
    pub fn UnregisterClassW(lpClassName: *const u16, hInstance: isize) -> i32;

    /// Calls the default window procedure for unhandled messages.
    ///
    /// # Returns
    ///
    /// The result of the default message processing.
    pub fn DefWindowProcW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> isize;

    /// Retrieves a message from the calling thread's message queue.
    ///
    /// # Returns
    ///
    /// Non-zero if a message other than `WM_QUIT` is retrieved.
    /// `0` if `WM_QUIT` is retrieved.
    /// `-1` on error.
    pub fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;

    /// Translates virtual-key messages into character messages.
    ///
    /// # Returns
    ///
    /// Non-zero if the message was translated; zero otherwise.
    pub fn TranslateMessage(lpMsg: *const MSG) -> i32;

    /// Dispatches a message to a window procedure.
    ///
    /// # Returns
    ///
    /// The value returned by the window procedure.
    pub fn DispatchMessageW(lpMsg: *const MSG) -> isize;

    /// Posts a `WM_QUIT` message to the calling thread's message queue.
    pub fn PostQuitMessage(nExitCode: i32);

    /// Creates a timer with the specified timeout.
    ///
    /// # Returns
    ///
    /// An identifier for the new timer, or `0` on failure.
    pub fn SetTimer(hWnd: HWND, nIDEvent: usize, uElapse: u32, lpTimerFunc: *const ()) -> usize;

    /// Destroys the specified timer.
    ///
    /// # Returns
    ///
    /// Non-zero on success; zero on failure.
    pub fn KillTimer(hWnd: HWND, uIDEvent: usize) -> i32;

    /// Retrieves or sets system-wide parameters (e.g., work area).
    ///
    /// # Returns
    ///
    /// Non-zero on success; zero on failure.
    pub fn SystemParametersInfoW(
        uiAction: u32,
        uiParam: u32,
        pvParam: *mut std::ffi::c_void,
        fWinIni: u32,
    ) -> i32;

    /// Sets the specified window's show state.
    ///
    /// # Returns
    ///
    /// Non-zero if the window was previously visible; zero if it was previously hidden.
    pub fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> i32;

    /// Retrieves the dimensions of the client area of a window.
    ///
    /// # Returns
    ///
    /// Non-zero on success; zero on failure.
    pub fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> i32;

    /// Changes the position and dimensions of a window.
    ///
    /// # Returns
    ///
    /// Non-zero on success; zero on failure.
    pub fn MoveWindow(hWnd: HWND, X: i32, Y: i32, nWidth: i32, nHeight: i32, bRepaint: i32) -> i32;

    /// Sets a value at the specified offset into the extra window memory.
    ///
    /// # Returns
    ///
    /// The previous value at the specified offset.
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: isize) -> isize;

    /// Retrieves a value at the specified offset into the extra window memory.
    ///
    /// # Returns
    ///
    /// The requested value, or `0` on failure.
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: i32) -> isize;

    /// Sends a message to a window and waits for the window procedure to process it.
    ///
    /// # Returns
    ///
    /// The result of the message processing, depending on the message sent.
    pub fn SendMessageW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> isize;

    /// Retrieves the DPI value for a specified window.
    ///
    /// # Returns
    ///
    /// The DPI value for the window, or `0` on failure.
    pub fn GetDpiForWindow(hwnd: HWND) -> u32;

    /// Sets the text of a window's title bar or control content.
    ///
    /// # Returns
    ///
    /// Non-zero on success; zero on failure.
    pub fn SetWindowTextW(hWnd: HWND, lpString: *const u16) -> i32;

    /// Sets the DPI awareness context for the calling thread.
    ///
    /// # Returns
    ///
    /// The previous DPI awareness context on success, or `0` on failure.
    pub fn SetThreadDpiAwarenessContext(dpiContext: isize) -> isize;

    /// Retrieves a module handle for the specified module.
    ///
    /// # Returns
    ///
    /// A handle to the specified module, or `0` on failure.
    pub fn GetModuleHandleW(lpModuleName: LPCWSTR) -> isize;
}

#[link(name = "Gdi32")]
unsafe extern "system" {
    /// Creates a logical font with the specified characteristics.
    ///
    /// # Returns
    ///
    /// A handle to the new font (HFONT), or `0` on failure.
    pub fn CreateFontW(
        cHeight: i32,
        cWidth: i32,
        cEscapement: i32,
        cOrientation: i32,
        cWeight: i32,
        bItalic: u32,
        bUnderline: u32,
        bStrikeOut: u32,
        iCharSet: u32,
        iOutPrecision: u32,
        iClipPrecision: u32,
        iQuality: u32,
        iPitchAndFamily: u32,
        pszFaceName: *const u16,
    ) -> isize;

    /// Deletes a logical pen, brush, font, bitmap, region, or palette.
    ///
    /// # Returns
    ///
    /// Non-zero on success; zero on failure.
    pub fn DeleteObject(ho: isize) -> i32;
}

#[link(name = "Shell32")]
unsafe extern "system" {
    /// Sends a notification message to the system taskbar's status area.
    ///
    /// # Parameters
    ///
    /// - `dwMessage`: The action to perform (NIM_ADD, NIM_MODIFY, NIM_DELETE).
    /// - `lpData`: Pointer to a `NOTIFYICONDATAW` structure.
    ///
    /// # Returns
    ///
    /// Non-zero on success; zero on failure.
    pub fn Shell_NotifyIconW(dwMessage: u32, lpData: *const NOTIFYICONDATAW) -> i32;
}
