use crate::{windows_api, wm_messages};
use serde::{Deserialize, Serialize};
use windows::Win32::UI::Input::KeyboardAndMouse::*;

#[derive(Deserialize, Serialize)]
struct UserVariantKeybind {
    index: usize,
    previous: String,
    next: String,
}

#[derive(Deserialize, Serialize)]
pub struct UserKeybinds {
    focus_previous: String,
    focus_next: String,
    swap_previous: String,
    swap_next: String,
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
    variant_keybinds: Vec<UserVariantKeybind>,
}

impl Default for UserKeybinds {
    fn default() -> Self {
        Self {
            focus_previous: "alt j".to_owned(),
            focus_next: "alt k".to_owned(),
            swap_previous: "alt shift j".to_owned(),
            swap_next: "alt shift k".to_owned(),
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
            variant_keybinds: vec![
                UserVariantKeybind {
                    index: 0,
                    previous: "alt h".to_owned(),
                    next: "alt l".to_owned(),
                },
                UserVariantKeybind {
                    index: 1,
                    previous: "alt shift h".to_owned(),
                    next: "alt shift l".to_owned(),
                },
            ],
        }
    }
}

struct Keybind {
    modifiers: HOT_KEY_MODIFIERS,
    key: u32,
}

impl TryFrom<&String> for Keybind {
    type Error = &'static str;

    fn try_from(value: &String) -> Result<Self, Self::Error> {
        let special_keys = std::collections::HashMap::from([("space", VK_SPACE.0 as u32)]);
        let lowercase = value.to_lowercase();
        let mut parsed_keys = lowercase
            .split(' ')
            .map(|w| w.trim())
            .filter(|w| w.len() > 0)
            .collect::<Vec<&str>>();
        let parsed_key = match parsed_keys.pop() {
            Some(k) => k,
            None => return Err(""),
        };
        if !special_keys.contains_key(&parsed_key) && parsed_key.len() > 1 {
            return Err("");
        }
        let parsed_modifiers = match parsed_keys.len() {
            0 => return Err(""),
            _ => parsed_keys,
        };
        let key = match special_keys.get(&parsed_key) {
            Some(k) => *k,
            None => match parsed_key.to_owned().to_uppercase().bytes().next() {
                Some(k) => k as u32,
                None => return Err(""),
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
                _ => return Err(""),
            }
        }
        return Ok(Keybind { modifiers, key });
    }
}

struct VariantKeybind {
    index: usize,
    previous: Keybind,
    next: Keybind,
}

impl TryFrom<&UserVariantKeybind> for VariantKeybind {
    type Error = &'static str;

    fn try_from(value: &UserVariantKeybind) -> Result<Self, Self::Error> {
        let previous = Keybind::try_from(&value.previous)?;
        let next = Keybind::try_from(&value.next)?;
        return Ok(Self {
            index: value.index,
            previous,
            next,
        });
    }
}

pub struct Keybinds {
    focus_previous: Result<Keybind, &'static str>,
    focus_next: Result<Keybind, &'static str>,
    swap_previous: Result<Keybind, &'static str>,
    swap_next: Result<Keybind, &'static str>,
    layout_previous: Result<Keybind, &'static str>,
    layout_next: Result<Keybind, &'static str>,
    focus_previous_monitor: Result<Keybind, &'static str>,
    focus_next_monitor: Result<Keybind, &'static str>,
    move_to_previous_monitor: Result<Keybind, &'static str>,
    move_to_next_monitor: Result<Keybind, &'static str>,
    grab_window: Result<Keybind, &'static str>,
    release_window: Result<Keybind, &'static str>,
    toggle_window: Result<Keybind, &'static str>,
    toggle_workspace: Result<Keybind, &'static str>,
    refresh_workspace: Result<Keybind, &'static str>,
    restart_himewm: Result<Keybind, &'static str>,
    variant_keybinds: std::collections::HashMap<usize, VariantKeybind>,
}

impl From<&UserKeybinds> for Keybinds {
    fn from(value: &UserKeybinds) -> Self {
        let mut variant_keybinds = std::collections::HashMap::new();
        for user_variant_keybind in &value.variant_keybinds {
            if let Ok(variant_keybind) = VariantKeybind::try_from(user_variant_keybind) {
                variant_keybinds.insert(variant_keybind.index, variant_keybind);
            }
        }
        return Self {
            focus_previous: Keybind::try_from(&value.focus_previous),
            focus_next: Keybind::try_from(&value.focus_next),
            swap_previous: Keybind::try_from(&value.swap_previous),
            swap_next: Keybind::try_from(&value.swap_next),
            layout_previous: Keybind::try_from(&value.layout_previous),
            layout_next: Keybind::try_from(&value.layout_next),
            focus_previous_monitor: Keybind::try_from(&value.focus_previous_monitor),
            focus_next_monitor: Keybind::try_from(&value.focus_next_monitor),
            move_to_previous_monitor: Keybind::try_from(&value.move_to_previous_monitor),
            move_to_next_monitor: Keybind::try_from(&value.move_to_next_monitor),
            grab_window: Keybind::try_from(&value.grab_window),
            release_window: Keybind::try_from(&value.release_window),
            toggle_window: Keybind::try_from(&value.toggle_window),
            toggle_workspace: Keybind::try_from(&value.toggle_workspace),
            refresh_workspace: Keybind::try_from(&value.refresh_workspace),
            restart_himewm: Keybind::try_from(&value.restart_himewm),
            variant_keybinds,
        };
    }
}

pub fn register_hotkeys(keybinds: Keybinds) {
    if let Ok(keybind) = keybinds.focus_previous {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::FOCUS_PREVIOUS as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Ok(keybind) = keybinds.focus_next {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::FOCUS_NEXT as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Ok(keybind) = keybinds.swap_previous {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::SWAP_PREVIOUS as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Ok(keybind) = keybinds.swap_next {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::SWAP_NEXT as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Ok(keybind) = keybinds.layout_previous {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::LAYOUT_PREVIOUS as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Ok(keybind) = keybinds.layout_next {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::LAYOUT_NEXT as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Ok(keybind) = keybinds.focus_previous_monitor {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::FOCUS_PREVIOUS_MONITOR as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Ok(keybind) = keybinds.focus_next_monitor {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::FOCUS_NEXT_MONITOR as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Ok(keybind) = keybinds.move_to_previous_monitor {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::MOVE_TO_PREVIOUS_MONITOR as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Ok(keybind) = keybinds.move_to_next_monitor {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::MOVE_TO_NEXT_MONITOR as i32,
            keybind.modifiers,
            keybind.key,
        );
    }
    if let Ok(keybind) = keybinds.grab_window {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::GRAB_WINDOW as i32,
            keybind.modifiers | MOD_NOREPEAT,
            keybind.key,
        );
    }
    if let Ok(keybind) = keybinds.release_window {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::RELEASE_WINDOW as i32,
            keybind.modifiers | MOD_NOREPEAT,
            keybind.key,
        );
    }
    if let Ok(keybind) = keybinds.toggle_window {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::TOGGLE_WINDOW as i32,
            keybind.modifiers | MOD_NOREPEAT,
            keybind.key,
        );
    }
    if let Ok(keybind) = keybinds.toggle_workspace {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::TOGGLE_WORKSPACE as i32,
            keybind.modifiers | MOD_NOREPEAT,
            keybind.key,
        );
    }
    if let Ok(keybind) = keybinds.refresh_workspace {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::REFRESH_WORKSPACE as i32,
            keybind.modifiers | MOD_NOREPEAT,
            keybind.key,
        );
    }
    if let Ok(keybind) = keybinds.restart_himewm {
        let _ = windows_api::register_hot_key(
            None,
            wm_messages::hotkey_identifiers::REQUEST_RESTART as i32,
            keybind.modifiers | MOD_NOREPEAT,
            keybind.key,
        );
    }
    for (key, variant_keybind) in keybinds.variant_keybinds {
        let _ = windows_api::register_hot_key(
            None,
            2 * (wm_messages::hotkey_identifiers::VARIANT_START + key) as i32,
            variant_keybind.previous.modifiers,
            variant_keybind.previous.key,
        );
        let _ = windows_api::register_hot_key(
            None,
            2 * (wm_messages::hotkey_identifiers::VARIANT_START + key) as i32 + 1,
            variant_keybind.next.modifiers,
            variant_keybind.next.key,
        );
    }
}
