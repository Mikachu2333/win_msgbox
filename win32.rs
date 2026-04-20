#[allow(clippy::upper_case_acronyms)]
pub type HWND = isize;
#[allow(clippy::upper_case_acronyms)]
pub type LPCWSTR = *const u16;
#[allow(clippy::upper_case_acronyms)]
pub type UINT = u32;
#[allow(clippy::upper_case_acronyms)]
pub type WORD = u16;
#[allow(clippy::upper_case_acronyms)]
pub type WPARAM = usize;
#[allow(clippy::upper_case_acronyms)]
pub type LPARAM = isize;
#[allow(clippy::upper_case_acronyms)]
pub type BOOL = i32;

pub const MB_SYSTEMMODAL: UINT = 0x1000;
pub const MB_SETFOREGROUND: UINT = 0x10000;
pub const WM_CLOSE: UINT = 0x0010;

pub const NIM_MODIFY: u32 = 0x00000001;
pub const NIF_INFO: u32 = 0x00000010;
pub const NIIF_INFO: u32 = 0x00000001;

pub const WS_POPUP: u32 = 0x80000000;
pub const WS_CAPTION: u32 = 0x00C00000;
pub const WS_SYSMENU: u32 = 0x00080000;
pub const WS_VISIBLE: u32 = 0x10000000;
pub const WS_CHILD: u32 = 0x40000000;
pub const WS_VSCROLL: u32 = 0x00200000;
pub const WS_EX_TOPMOST: u32 = 0x00000008;
pub const WS_EX_NOACTIVATE: u32 = 0x08000000;
pub const WS_EX_TOOLWINDOW: u32 = 0x00000080;

pub const ES_MULTILINE: u32 = 0x0004;
pub const ES_READONLY: u32 = 0x0800;
pub const ES_AUTOVSCROLL: u32 = 0x0040;

pub const WM_CREATE: u32 = 0x0001;
pub const WM_DESTROY: u32 = 0x0002;
pub const WM_TIMER: u32 = 0x0113;
pub const WM_SETFONT: u32 = 0x0030;
pub const WM_NCDESTROY: u32 = 0x0082;

pub const SPI_GETWORKAREA: u32 = 0x0030;
pub const GWLP_USERDATA: i32 = -21;

pub const FW_NORMAL: i32 = 400;
pub const DEFAULT_CHARSET: u32 = 1;
pub const OUT_DEFAULT_PRECIS: u32 = 0;
pub const CLIP_DEFAULT_PRECIS: u32 = 0;
pub const CLEARTYPE_QUALITY: u32 = 5;
pub const VARIABLE_PITCH: u32 = 2;
pub const FF_SWISS: u32 = 0x20;

pub const SW_SHOWNA: i32 = 8;
pub const DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2: isize = -4;

#[repr(C)]
#[derive(Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub struct RECT {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[repr(C)]
#[allow(non_snake_case, clippy::upper_case_acronyms)]
pub struct WNDCLASSEXW {
    pub cbSize: u32,
    pub style: u32,
    pub lpfnWndProc: unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> isize,
    pub cbClsExtra: i32,
    pub cbWndExtra: i32,
    pub hInstance: isize,
    pub hIcon: isize,
    pub hCursor: isize,
    pub hbrBackground: isize,
    pub lpszMenuName: *const u16,
    pub lpszClassName: *const u16,
    pub hIconSm: isize,
}

#[repr(C)]
#[allow(non_snake_case, clippy::upper_case_acronyms)]
pub struct MSG {
    pub hwnd: HWND,
    pub message: u32,
    pub wParam: WPARAM,
    pub lParam: LPARAM,
    pub time: u32,
    pub pt_x: i32,
    pub pt_y: i32,
}

#[repr(C)]
#[allow(non_snake_case, clippy::upper_case_acronyms)]
pub struct NOTIFYICONDATAW {
    pub cbSize: u32,
    pub hWnd: HWND,
    pub uID: UINT,
    pub uFlags: UINT,
    pub uCallbackMessage: UINT,
    pub hIcon: isize,
    pub szTip: [u16; 128],
    pub dwState: u32,
    pub dwStateMask: u32,
    pub szInfo: [u16; 256],
    pub uTimeoutOrVersion: UINT,
    pub szInfoTitle: [u16; 64],
    pub dwInfoFlags: u32,
    pub guidItem: [u8; 16],
    pub hBalloonIcon: isize,
}

#[link(name = "User32")]
unsafe extern "system" {
    pub fn MessageBoxExW(
        hWnd: HWND,
        lpText: LPCWSTR,
        lpCaption: LPCWSTR,
        uType: UINT,
        wLanguageId: WORD,
    ) -> i32;
    pub fn FindWindowW(lpClassName: LPCWSTR, lpWindowName: LPCWSTR) -> HWND;
    pub fn PostMessageW(hWnd: HWND, Msg: UINT, wParam: WPARAM, lParam: LPARAM) -> BOOL;
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
    pub fn DestroyWindow(hWnd: HWND) -> i32;

    pub fn RegisterClassExW(lpWndClass: *const WNDCLASSEXW) -> u16;
    pub fn UnregisterClassW(lpClassName: *const u16, hInstance: isize) -> i32;
    pub fn DefWindowProcW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> isize;
    pub fn GetMessageW(lpMsg: *mut MSG, hWnd: HWND, wMsgFilterMin: u32, wMsgFilterMax: u32) -> i32;
    pub fn TranslateMessage(lpMsg: *const MSG) -> i32;
    pub fn DispatchMessageW(lpMsg: *const MSG) -> isize;
    pub fn PostQuitMessage(nExitCode: i32);
    pub fn SetTimer(hWnd: HWND, nIDEvent: usize, uElapse: u32, lpTimerFunc: *const ()) -> usize;
    pub fn KillTimer(hWnd: HWND, uIDEvent: usize) -> i32;
    pub fn SystemParametersInfoW(
        uiAction: u32,
        uiParam: u32,
        pvParam: *mut std::ffi::c_void,
        fWinIni: u32,
    ) -> i32;
    pub fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> i32;
    pub fn GetClientRect(hWnd: HWND, lpRect: *mut RECT) -> i32;
    pub fn MoveWindow(hWnd: HWND, X: i32, Y: i32, nWidth: i32, nHeight: i32, bRepaint: i32) -> i32;
    pub fn SetWindowLongPtrW(hWnd: HWND, nIndex: i32, dwNewLong: isize) -> isize;
    pub fn GetWindowLongPtrW(hWnd: HWND, nIndex: i32) -> isize;
    pub fn SendMessageW(hWnd: HWND, Msg: u32, wParam: WPARAM, lParam: LPARAM) -> isize;
    pub fn GetDpiForWindow(hwnd: HWND) -> u32;
    pub fn SetWindowTextW(hWnd: HWND, lpString: *const u16) -> i32;
    pub fn SetThreadDpiAwarenessContext(dpiContext: isize) -> isize;
    pub fn GetModuleHandleW(lpModuleName: LPCWSTR) -> isize;
}

#[link(name = "Gdi32")]
unsafe extern "system" {
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
    pub fn DeleteObject(ho: isize) -> i32;
}

#[link(name = "Shell32")]
unsafe extern "system" {
    pub fn Shell_NotifyIconW(dwMessage: u32, lpData: *const NOTIFYICONDATAW) -> i32;
}
