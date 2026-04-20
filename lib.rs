/// Windows message box and notification module.
mod msgbox;
mod popup_notify;
mod tray_notify;
mod util;
mod win32;

pub use msgbox::{
    error_msgbox, info_msgbox, quest_msgbox_okcancel, quest_msgbox_yesno, warn_msgbox,
};
pub use popup_notify::{notify_msgbox_standalone, wait_notifications};
pub use tray_notify::{NotifyIconType, notify_msgbox};
pub use win32::HWND;
