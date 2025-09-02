use crate::{windows_api, wm_messages};
use serde::{Deserialize, Serialize};
use windows::Win32::UI::Input::KeyboardAndMouse::*;

#[derive(Deserialize, Serialize)]
pub struct UserKeybinds {
    focus_previous: String,
    focus_next: String,
    swap_previous: String,
    swap_next: String,
    variant_previous: String,
    variant_next: String,
    layout_previous: String,
    layout_next: String,
    focus_previous_monitor: String,
    focus_next_monitor: String,
    move_to_previous_monitor: String,
    move_to_next_monitor: String,
    grab_window: String,
    release_window: String,
    toggle_window: String,
    toggle_workspace: String,
    refresh_workspace: String,
    restart_himewm: String,
}

impl Default for UserKeybinds {
    fn default() -> Self {
        Self {
            focus_previous: "alt j".to_owned(),
            focus_next: "alt k".to_owned(),
            swap_previous: "alt shift j".to_owned(),
            swap_next: "alt shift k".to_owned(),
            variant_previous: "alt h".to_owned(),
            variant_next: "alt l".to_owned(),
            layout_previous: "alt y".to_owned(),
            layout_next: "alt o".to_owned(),
            focus_previous_monitor: "alt u".to_owned(),
            focus_next_monitor: "alt i".to_owned(),
            move_to_previous_monitor: "alt shift u".to_owned(),
            move_to_next_monitor: "alt shift i".to_owned(),
            grab_window: "alt p".to_owned(),
            release_window: "alt shift p".to_owned(),
            toggle_window: "alt shift space".to_owned(),
            toggle_workspace: "alt n".to_owned(),
            refresh_workspace: "alt r".to_owned(),
            restart_himewm: "alt shift r".to_owned(),
        }
    }
}

struct Keybind {
    modifiers: HOT_KEY_MODIFIERS,
    key: u32,
}

pub struct Keybinds {
    focus_previous: Option<Keybind>,
    focus_next: Option<Keybind>,
    swap_previous: Option<Keybind>,
    swap_next: Option<Keybind>,
    variant_previous: Option<Keybind>,
    variant_next: Option<Keybind>,
    layout_previous: Option<Keybind>,
    layout_next: Option<Keybind>,
    focus_previous_monitor: Option<Keybind>,
    focus_next_monitor: Option<Keybind>,
    move_to_previous_monitor: Option<Keybind>,
    move_to_next_monitor: Option<Keybind>,
    grab_window: Option<Keybind>,
    release_window: Option<Keybind>,
    toggle_window: Option<Keybind>,
    toggle_workspace: Option<Keybind>,
    refresh_workspace: Option<Keybind>,
    restart_himewm: Option<Keybind>,
}

impl From<&UserKeybinds> for Keybinds {
    fn from(value: &UserKeybinds) -> Self {
        Self {
            focus_previous: parse_keybind(&value.focus_previous),
            focus_next: parse_keybind(&value.focus_next),
            swap_previous: parse_keybind(&value.swap_previous),
            swap_next: parse_keybind(&value.swap_next),
            variant_previous: parse_keybind(&value.variant_previous),
            variant_next: parse_keybind(&value.variant_next),
            layout_previous: parse_keybind(&value.layout_previous),
            layout_next: parse_keybind(&value.layout_next),
            focus_previous_monitor: parse_keybind(&value.focus_previous_monitor),
            focus_next_monitor: parse_keybind(&value.focus_next_monitor),
            move_to_previous_monitor: parse_keybind(&value.move_to_previous_monitor),
            move_to_next_monitor: parse_keybind(&value.move_to_next_monitor),
            grab_window: parse_keybind(&value.grab_window),
            release_window: parse_keybind(&value.release_window),
            toggle_window: parse_keybind(&value.toggle_window),
            toggle_workspace: parse_keybind(&value.toggle_workspace),
            refresh_workspace: parse_keybind(&value.refresh_workspace),
            restart_himewm: parse_keybind(&value.restart_himewm),
        }
    }
}

fn parse_keybind(s: &String) -> Option<Keybind> {
    let special_keys = std::collections::HashMap::from([("space", VK_SPACE.0 as u32)]);
    let lowercase = s.to_lowercase();
    let mut parsed_keys = lowercase
        .split(' ')
        .map(|w| w.trim())
        .filter(|w| w.len() > 0)
        .collect::<Vec<&str>>();
    let parsed_key = match parsed_keys.pop() {
        Some(k) => k,
        None => return None,
    };
    if !special_keys.contains_key(&parsed_key) && parsed_key.len() > 1 {
        return None;
    }
    let parsed_modifiers = match parsed_keys.len() {
        0 => return None,
        _ => parsed_keys,
    };
    let key = match special_keys.get(&parsed_key) {
        Some(k) => *k,
        None => match parsed_key.to_owned().to_uppercase().bytes().next() {
            Some(k) => k as u32,
            None => return None,
        },
    };
    let mut modifiers = HOT_KEY_MODIFIERS(0);
    for modifier in parsed_modifiers {
        match modifier {
            "alt" => {
                modifiers |= MOD_ALT;
            }
            "ctrl" => {
                modifiers |= MOD_CONTROL;
            }
            "shift" => {
                modifiers |= MOD_SHIFT;
            }
            "win" => {
                modifiers |= MOD_WIN;
            }
            _ => return None,
        }
    }
    return Some(Keybind { modifiers, key });
}

pub fn register_hotkeys(keybinds: Keybinds) {
    if let Some(keybind) = keybinds.focus_previous {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::FOCUS_PREVIOUS as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Some(keybind) = keybinds.focus_next {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::FOCUS_NEXT as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Some(keybind) = keybinds.swap_previous {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::SWAP_PREVIOUS as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Some(keybind) = keybinds.swap_next {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::SWAP_NEXT as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Some(keybind) = keybinds.variant_previous {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::VARIANT_PREVIOUS as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Some(keybind) = keybinds.variant_next {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::VARIANT_NEXT as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Some(keybind) = keybinds.layout_previous {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::LAYOUT_PREVIOUS as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Some(keybind) = keybinds.layout_next {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::LAYOUT_NEXT as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Some(keybind) = keybinds.focus_previous_monitor {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::FOCUS_PREVIOUS_MONITOR as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Some(keybind) = keybinds.focus_next_monitor {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::FOCUS_NEXT_MONITOR as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Some(keybind) = keybinds.move_to_previous_monitor {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::MOVE_TO_PREVIOUS_MONITOR as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Some(keybind) = keybinds.move_to_next_monitor {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::MOVE_TO_NEXT_MONITOR as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Some(keybind) = keybinds.grab_window {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::GRAB_WINDOW as i32,
            keybind.modifiers | MOD_NOREPEAT,
            keybind.key,
        );
    }
    if let Some(keybind) = keybinds.release_window {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::RELEASE_WINDOW as i32,
            keybind.modifiers | MOD_NOREPEAT,
            keybind.key,
        );
    }
    if let Some(keybind) = keybinds.toggle_window {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::TOGGLE_WINDOW as i32,
            keybind.modifiers | MOD_NOREPEAT,
            keybind.key,
        );
    }
    if let Some(keybind) = keybinds.toggle_workspace {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::TOGGLE_WORKSPACE as i32,
            keybind.modifiers | MOD_NOREPEAT,
            keybind.key,
        );
    }
    if let Some(keybind) = keybinds.refresh_workspace {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::REFRESH_WORKSPACE as i32,
            keybind.modifiers | MOD_NOREPEAT,
            keybind.key,
        );
    }
    if let Some(keybind) = keybinds.restart_himewm {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::RESTART_HIMEWM as i32,
            keybind.modifiers | MOD_NOREPEAT,
            keybind.key,
        );
    }
}
