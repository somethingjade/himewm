use crate::windows_api;
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

pub enum MessageType {
    None,
    Warning,
    Error,
}

pub fn add_to_message(message_buffer: &mut String, message: &str) {
    if !message_buffer.is_empty() {
        message_buffer.push_str("\n\n");
    }
    message_buffer.push_str(message);
}

pub fn get_console_hwnd() -> HWND {
    let console_hwnd = windows_api::get_console_window();
    let _set_foreground_window = windows_api::set_foreground_window(console_hwnd);
    return windows_api::get_foreground_window();
}

pub fn display_message(console_hwnd: HWND, message_type: &MessageType, message: &str) {
    if let MessageType::None = message_type {
        return;
    }
    let _clear_console_window = std::process::Command::new("cmd")
        .args(["/c", "cls"])
        .status();
    let _show_console_window = windows_api::show_window(console_hwnd, SW_SHOW);
    let _set_foreground_window = windows_api::set_foreground_window(console_hwnd);
    println!("{}", message);
    let prompt_str = match message_type {
        MessageType::Error => "Press ENTER to exit",
        _ => "Press ENTER to continue",
    };
    println!("");
    println!("{}", prompt_str);
    let mut buf = String::new();
    let _read_line = std::io::stdin().read_line(&mut buf);
    let _hide_console_window = windows_api::show_window(console_hwnd, SW_HIDE);
}
