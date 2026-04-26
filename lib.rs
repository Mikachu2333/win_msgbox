//! Windows message box and notification module.
//!
//! This crate provides utility functions for displaying native Windows message boxes
//! and system notifications with support for auto-close timeout, multiple icon styles,
//! and various button combinations.
//!
//! # Features
//!
//! - **Message boxes**: Native Win32 message boxes with icon and button presets,
//!   plus optional auto-close timeout.
//! - **Standalone popup notification**: A custom popup window in the bottom-right
//!   corner with live countdown display.
//! - **Tray balloon helper**: Display balloon notifications on existing system tray icons.
//! - **C/C++ FFI**: Expose `custom_msgbox_w` for direct calling from C/C++ code
//!   (requires `cdylib` crate type).
//!
//! # Quick Start
//!
//! ```rust
//! use win_msgbox_timeout::{
//!     MsgBoxType, MsgBtnType, custom_msgbox, error_msgbox, info_msgbox,
//!     quest_msgbox_yesno, warn_msgbox,
//! };
//!
//! // Simple info dialog
//! info_msgbox("Hello from Rust", "Info", 0);
//!
//! // Warning with 3-second auto-close
//! warn_msgbox("Careful!", "Warning", 3000);
//!
//! // Yes/No question
//! let result = quest_msgbox_yesno("Do you want to continue?", "Question", 0);
//! if result == 6 {
//!     info_msgbox("You clicked Yes", "Result", 0);
//! }
//! ```
//!
//! # C/C++ Interop
//!
//! When built with `crate-type = ["cdylib"]`, the crate exports `custom_msgbox_w`
//! as a C-compatible function. See the [`custom_msgbox_w`] documentation for details.

mod msgbox;
mod c_msgbox;
mod popup_notify;
mod tray_notify;
mod util;
mod win32;

pub use msgbox::{
    MsgBoxType, MsgBtnType, custom_msgbox, error_msgbox, info_msgbox, quest_msgbox_okcancel,
    quest_msgbox_yesno, warn_msgbox,
};
pub use c_msgbox::custom_msgbox_w;
pub use popup_notify::{notify_msgbox_standalone, wait_notifications};
pub use tray_notify::{NotifyIconType, notify_msgbox};
pub use win32::HWND;
