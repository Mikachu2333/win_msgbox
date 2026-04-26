# win_msgbox

A small Rust library for native Windows message boxes and notification popups.

## Features

- Native Win32 message boxes with icon and button presets
- Optional auto-close timeout for message boxes
- Standalone popup notification window (bottom-right corner)
- Balloon tip helper for existing tray icons

## Platform

- Windows only
- Uses raw Win32 FFI (User32, Shell32, Gdi32)

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
win_msgbox = "1.1.0"
```

## Quick Start

```rust
use win_msgbox::{
    MsgBoxType, MsgBtnType, custom_msgbox, error_msgbox, info_msgbox, notify_msgbox_standalone,
    quest_msgbox_yesno, wait_notifications, warn_msgbox,
};

fn main() {
    info_msgbox("Hello from Rust", "Info", 0);
    warn_msgbox("Careful!", "Warning", 3000);

    let result = quest_msgbox_yesno("Do you want to continue?", "Question", 0);
    if result == 6 {
        info_msgbox("You clicked Yes", "Result", 0);
    } else if result == 7 {
        error_msgbox("You clicked No", "Result", 0);
    }

    custom_msgbox(
        "Custom buttons and icon",
        "Custom",
        MsgBoxType::Warn,
        MsgBtnType::OkCancel,
        0,
    );

    notify_msgbox_standalone("Task", "Operation completed", 5000);

    // Wait for standalone notifications before process exit.
    wait_notifications();
}
```

## API Overview

### Message Boxes

- `info_msgbox(msg, title, timeout_ms) -> i32`
- `error_msgbox(msg, title, timeout_ms) -> i32`
- `warn_msgbox(msg, title, timeout_ms) -> i32`
- `quest_msgbox_yesno(msg, title, timeout_ms) -> i32`
- `quest_msgbox_okcancel(msg, title, timeout_ms) -> i32`
- `custom_msgbox(msg, title, msgbox_type, msgboxbtn_type, timeout_ms) -> i32`

Return values follow Win32 `MessageBox` conventions:

- `1`: OK
- `2`: Cancel
- `6`: Yes
- `7`: No
- `-1`: closed by timeout

### Standalone Notification Popup

- `notify_msgbox_standalone(title, msg, timeout_ms) -> bool`
- `wait_notifications()`

### Tray Balloon Helper

- `notify_msgbox(hwnd, msg, icon_id) -> i32`

Notes:

- `notify_msgbox` requires an existing tray icon created with the same `icon_id`.
- `NotifyIconType` enum is exported for future extension.

## Publish to crates.io

1. Create a crates.io account and API token.
2. Log in from terminal:

    ```powershell
    cargo login <YOUR_CRATES_IO_TOKEN>
    ```

3. Validate package locally:

    ```powershell
    cargo fmt --check
    cargo clippy -- -D warnings
    cargo package
    ```

4. Publish:

    ```powershell
    cargo publish
    ```

5. Verify:

    ```powershell
    cargo search win_msgbox
    ```

## Notes Before Publishing

- Ensure the version in `Cargo.toml` is new (cannot republish same version).
- Make sure `license`, `description`, and `readme` are present.
- Consider adding `repository` when you have a public Git repository URL.

## License

MIT
