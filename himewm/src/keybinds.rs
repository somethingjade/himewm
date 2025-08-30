use crate::windows_api;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

pub mod hotkey_identifiers {
    pub const FOCUS_PREVIOUS: usize = 0;
    pub const FOCUS_NEXT: usize = 1;
    pub const SWAP_PREVIOUS: usize = 2;
    pub const SWAP_NEXT: usize = 3;
    pub const VARIANT_PREVIOUS: usize = 4;
    pub const VARIANT_NEXT: usize = 5;
    pub const LAYOUT_PREVIOUS: usize = 6;
    pub const LAYOUT_NEXT: usize = 7;
    pub const FOCUS_PREVIOUS_MONITOR: usize = 8;
    pub const FOCUS_NEXT_MONITOR: usize = 9;
    pub const MOVE_TO_PREVIOUS_MONITOR: usize = 10;
    pub const MOVE_TO_NEXT_MONITOR: usize = 11;
    pub const GRAB_WINDOW: usize = 12;
    pub const RELEASE_WINDOW: usize = 13;
    pub const TOGGLE_WINDOW: usize = 14;
    pub const TOGGLE_WORKSPACE: usize = 15;
}

pub fn register_hotkeys() {
    let _focus_previous = windows_api::register_hot_key(
        None,
        hotkey_identifiers::FOCUS_PREVIOUS as i32,
        MOD_ALT,
        0x4A,
    );
    let _focus_next =
        windows_api::register_hot_key(None, hotkey_identifiers::FOCUS_NEXT as i32, MOD_ALT, 0x4B);
    let _swap_previous = windows_api::register_hot_key(
        None,
        hotkey_identifiers::SWAP_PREVIOUS as i32,
        MOD_ALT,
        0x48,
    );
    let _swap_next =
        windows_api::register_hot_key(None, hotkey_identifiers::SWAP_NEXT as i32, MOD_ALT, 0x4C);
    let _variant_previous = windows_api::register_hot_key(
        None,
        hotkey_identifiers::VARIANT_PREVIOUS as i32,
        MOD_ALT | MOD_SHIFT,
        0x4A,
    );
    let _variant_next = windows_api::register_hot_key(
        None,
        hotkey_identifiers::VARIANT_NEXT as i32,
        MOD_ALT | MOD_SHIFT,
        0x4B,
    );
    let _layout_previous = windows_api::register_hot_key(
        None,
        hotkey_identifiers::LAYOUT_PREVIOUS as i32,
        MOD_ALT | MOD_SHIFT,
        0x48,
    );
    let _layout_next = windows_api::register_hot_key(
        None,
        hotkey_identifiers::LAYOUT_NEXT as i32,
        MOD_ALT | MOD_SHIFT,
        0x4C,
    );
    let _focus_previous_monitor = windows_api::register_hot_key(
        None,
        hotkey_identifiers::FOCUS_PREVIOUS_MONITOR as i32,
        MOD_ALT,
        0x55,
    );
    let _focus_next_monitor = windows_api::register_hot_key(
        None,
        hotkey_identifiers::FOCUS_NEXT_MONITOR as i32,
        MOD_ALT,
        0x49,
    );
    let _move_to_previous_monitor = windows_api::register_hot_key(
        None,
        hotkey_identifiers::MOVE_TO_PREVIOUS_MONITOR as i32,
        MOD_ALT,
        0x59,
    );
    let _move_to_next_monitor = windows_api::register_hot_key(
        None,
        hotkey_identifiers::MOVE_TO_NEXT_MONITOR as i32,
        MOD_ALT,
        0x4F,
    );
    let _grab_window = windows_api::register_hot_key(
        None,
        hotkey_identifiers::GRAB_WINDOW as i32,
        MOD_ALT | MOD_SHIFT | MOD_NOREPEAT,
        0x55,
    );
    let _release_window = windows_api::register_hot_key(
        None,
        hotkey_identifiers::RELEASE_WINDOW as i32,
        MOD_ALT | MOD_SHIFT | MOD_NOREPEAT,
        0x49,
    );
    let _toggle_window = windows_api::register_hot_key(
        None,
        hotkey_identifiers::TOGGLE_WINDOW as i32,
        MOD_ALT | MOD_SHIFT | MOD_NOREPEAT,
        0x59,
    );
    let _toggle_workspace = windows_api::register_hot_key(
        None,
        hotkey_identifiers::TOGGLE_WORKSPACE as i32,
        MOD_ALT | MOD_SHIFT | MOD_NOREPEAT,
        0x4F,
    );
}
