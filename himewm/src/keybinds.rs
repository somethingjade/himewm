use crate::{util, windows_api, wm};
use serde::{Deserialize, Serialize};
use windows::Win32::UI::Input::KeyboardAndMouse::*;

#[derive(Deserialize, Serialize)]
struct UserVariantKeybinds {
    previous: String,
    next: String,
}

#[derive(Deserialize, Serialize)]
struct UserVariantKeybind {
    index: usize,
    keybinds: UserVariantKeybinds,
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
                    keybinds: UserVariantKeybinds {
                        previous: "alt h".to_owned(),
                        next: "alt l".to_owned(),
                    },
                },
                UserVariantKeybind {
                    index: 1,
                    keybinds: UserVariantKeybinds {
                        previous: "alt shift h".to_owned(),
                        next: "alt shift l".to_owned(),
                    },
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
            .filter(|w| !w.is_empty())
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

struct VariantKeybinds {
    previous: Keybind,
    next: Keybind,
}

struct VariantKeybind {
    index: usize,
    keybinds: VariantKeybinds,
}

impl TryFrom<&UserVariantKeybind> for VariantKeybind {
    type Error = &'static str;

    fn try_from(value: &UserVariantKeybind) -> Result<Self, Self::Error> {
        let previous = Keybind::try_from(&value.keybinds.previous)?;
        let next = Keybind::try_from(&value.keybinds.next)?;
        return Ok(Self {
            index: value.index,
            keybinds: VariantKeybinds { previous, next },
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
    variant_keybinds: Vec<VariantKeybind>,
}

impl From<&UserKeybinds> for Keybinds {
    fn from(value: &UserKeybinds) -> Self {
        let mut variant_keybinds = Vec::new();
        for user_variant_keybind in &value.variant_keybinds {
            if let Ok(variant_keybind) = VariantKeybind::try_from(user_variant_keybind) {
                variant_keybinds.push(variant_keybind);
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

pub fn register_hotkeys(keybinds: &Keybinds, warnings_string: &mut String) {
    if let Ok(keybind) = &keybinds.focus_previous {
        if let Err(e) = windows_api::register_hot_key(
            None,
            wm::messages::hotkey_identifiers::FOCUS_PREVIOUS as i32,
            keybind.modifiers,
            keybind.key,
        ) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to register focus_previous hotkey\n{}",
                    e.message()
                ),
            );
        }
    }
    if let Ok(keybind) = &keybinds.focus_next {
        if let Err(e) = windows_api::register_hot_key(
            None,
            wm::messages::hotkey_identifiers::FOCUS_NEXT as i32,
            keybind.modifiers,
            keybind.key,
        ) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to register focus_next hotkey\n{}",
                    e.message()
                ),
            );
        }
    }
    if let Ok(keybind) = &keybinds.swap_previous {
        if let Err(e) = windows_api::register_hot_key(
            None,
            wm::messages::hotkey_identifiers::SWAP_PREVIOUS as i32,
            keybind.modifiers,
            keybind.key,
        ) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to register swap_previous hotkey\n{}",
                    e.message()
                ),
            );
        }
    }
    if let Ok(keybind) = &keybinds.swap_next {
        if let Err(e) = windows_api::register_hot_key(
            None,
            wm::messages::hotkey_identifiers::SWAP_NEXT as i32,
            keybind.modifiers,
            keybind.key,
        ) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to register swap_next hotkey\n{}",
                    e.message()
                ),
            );
        }
    }
    if let Ok(keybind) = &keybinds.layout_previous {
        if let Err(e) = windows_api::register_hot_key(
            None,
            wm::messages::hotkey_identifiers::LAYOUT_PREVIOUS as i32,
            keybind.modifiers,
            keybind.key,
        ) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to register layout_previous hotkey\n{}",
                    e.message()
                ),
            );
        }
    }
    if let Ok(keybind) = &keybinds.layout_next {
        if let Err(e) = windows_api::register_hot_key(
            None,
            wm::messages::hotkey_identifiers::LAYOUT_NEXT as i32,
            keybind.modifiers,
            keybind.key,
        ) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to register layout_next hotkey\n{}",
                    e.message()
                ),
            );
        }
    }
    if let Ok(keybind) = &keybinds.focus_previous_monitor {
        if let Err(e) = windows_api::register_hot_key(
            None,
            wm::messages::hotkey_identifiers::FOCUS_PREVIOUS_MONITOR as i32,
            keybind.modifiers,
            keybind.key,
        ) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to register focus_previous_monitor hotkey\n{}",
                    e.message()
                ),
            );
        }
    }
    if let Ok(keybind) = &keybinds.focus_next_monitor {
        if let Err(e) = windows_api::register_hot_key(
            None,
            wm::messages::hotkey_identifiers::FOCUS_NEXT_MONITOR as i32,
            keybind.modifiers,
            keybind.key,
        ) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to register focus_next_monitor hotkey\n{}",
                    e.message()
                ),
            );
        }
    }
    if let Ok(keybind) = &keybinds.move_to_previous_monitor {
        if let Err(e) = windows_api::register_hot_key(
            None,
            wm::messages::hotkey_identifiers::MOVE_TO_PREVIOUS_MONITOR as i32,
            keybind.modifiers,
            keybind.key,
        ) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to register move_to_previous_monitor hotkey\n{}",
                    e.message()
                ),
            );
        }
    }
    if let Ok(keybind) = &keybinds.move_to_next_monitor {
        if let Err(e) = windows_api::register_hot_key(
            None,
            wm::messages::hotkey_identifiers::MOVE_TO_NEXT_MONITOR as i32,
            keybind.modifiers,
            keybind.key,
        ) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to register move_to_next_monitor hotkey\n{}",
                    e.message()
                ),
            );
        }
    }
    if let Ok(keybind) = &keybinds.grab_window {
        if let Err(e) = windows_api::register_hot_key(
            None,
            wm::messages::hotkey_identifiers::GRAB_WINDOW as i32,
            keybind.modifiers | MOD_NOREPEAT,
            keybind.key,
        ) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to register grab_window hotkey\n{}",
                    e.message()
                ),
            );
        }
    }
    if let Ok(keybind) = &keybinds.release_window {
        if let Err(e) = windows_api::register_hot_key(
            None,
            wm::messages::hotkey_identifiers::RELEASE_WINDOW as i32,
            keybind.modifiers | MOD_NOREPEAT,
            keybind.key,
        ) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to register release_window hotkey\n{}",
                    e.message()
                ),
            );
        }
    }
    if let Ok(keybind) = &keybinds.toggle_window {
        if let Err(e) = windows_api::register_hot_key(
            None,
            wm::messages::hotkey_identifiers::TOGGLE_WINDOW as i32,
            keybind.modifiers | MOD_NOREPEAT,
            keybind.key,
        ) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to register toggle_window hotkey\n{}",
                    e.message()
                ),
            );
        }
    }
    if let Ok(keybind) = &keybinds.toggle_workspace {
        if let Err(e) = windows_api::register_hot_key(
            None,
            wm::messages::hotkey_identifiers::TOGGLE_WORKSPACE as i32,
            keybind.modifiers | MOD_NOREPEAT,
            keybind.key,
        ) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to register toggle_workspace hotkey\n{}",
                    e.message()
                ),
            );
        }
    }
    if let Ok(keybind) = &keybinds.refresh_workspace {
        if let Err(e) = windows_api::register_hot_key(
            None,
            wm::messages::hotkey_identifiers::REFRESH_WORKSPACE as i32,
            keybind.modifiers | MOD_NOREPEAT,
            keybind.key,
        ) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to register refresh_workspace hotkey\n{}",
                    e.message()
                ),
            );
        }
    }
    if let Ok(keybind) = &keybinds.restart_himewm {
        if let Err(e) = windows_api::register_hot_key(
            None,
            wm::messages::hotkey_identifiers::REQUEST_RESTART as i32,
            keybind.modifiers | MOD_NOREPEAT,
            keybind.key,
        ) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to register restart_himewm hotkey\n{}",
                    e.message()
                ),
            );
        }
    }
    for variant_keybind in &keybinds.variant_keybinds {
        if let Err(e) = windows_api::register_hot_key(
            None,
            (wm::messages::hotkey_identifiers::VARIANT_START + 2 * variant_keybind.index) as i32,
            variant_keybind.keybinds.previous.modifiers,
            variant_keybind.keybinds.previous.key,
        ) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to register variant_keybinds hotkey (index: {}, previous)\n{}",
                    variant_keybind.index,
                    e.message()
                ),
            );
        }
        if let Err(e) = windows_api::register_hot_key(
            None,
            (wm::messages::hotkey_identifiers::VARIANT_START + 2 * variant_keybind.index + 1)
                as i32,
            variant_keybind.keybinds.next.modifiers,
            variant_keybind.keybinds.next.key,
        ) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to register variant_keybinds hotkey (index: {}, next)\n{}",
                    variant_keybind.index,
                    e.message()
                ),
            );
        }
    }
}

pub fn unregister_hotkeys(keybinds: Keybinds, warnings_string: &mut String) {
    for id in wm::messages::hotkey_identifiers::HOTKEY_IDENTIFIERS_START
        ..wm::messages::hotkey_identifiers::VARIANT_START
    {
        if let Err(e) = windows_api::unregister_hot_key(None, id as i32) {
            let hotkey_str = match id {
                wm::messages::hotkey_identifiers::FOCUS_PREVIOUS => "focus_previous",
                wm::messages::hotkey_identifiers::FOCUS_NEXT => "focus_next",
                wm::messages::hotkey_identifiers::SWAP_PREVIOUS => "swap_previous",
                wm::messages::hotkey_identifiers::SWAP_NEXT => "swap_next",
                wm::messages::hotkey_identifiers::LAYOUT_PREVIOUS => "layout_previous",
                wm::messages::hotkey_identifiers::LAYOUT_NEXT => "layout_next",
                wm::messages::hotkey_identifiers::FOCUS_PREVIOUS_MONITOR => {
                    "focus_previous_monitor"
                }
                wm::messages::hotkey_identifiers::FOCUS_NEXT_MONITOR => "focus_next_monitor",
                wm::messages::hotkey_identifiers::MOVE_TO_PREVIOUS_MONITOR => {
                    "move_to_previous_monitor"
                }
                wm::messages::hotkey_identifiers::MOVE_TO_NEXT_MONITOR => "move_to_next_monitor",
                wm::messages::hotkey_identifiers::GRAB_WINDOW => "grab_window",
                wm::messages::hotkey_identifiers::RELEASE_WINDOW => "release_window",
                wm::messages::hotkey_identifiers::TOGGLE_WINDOW => "toggle_window",
                wm::messages::hotkey_identifiers::TOGGLE_WORKSPACE => "toggle_workspace",
                wm::messages::hotkey_identifiers::REFRESH_WORKSPACE => "refresh_workspace",
                wm::messages::hotkey_identifiers::REQUEST_RESTART => "request_restart",
                _ => continue,
            };
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: failed to unregister {} hotkey\n{}",
                    hotkey_str,
                    e.message()
                ),
            );
        }
    }
    for variant_keybind in keybinds.variant_keybinds {
        let previous_id =
            (wm::messages::hotkey_identifiers::VARIANT_START + 2 * variant_keybind.index) as i32;
        if let Err(e) = windows_api::unregister_hot_key(None, previous_id) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to unregister variant_keybinds hotkey (index: {}, previous)\n{}",
                    variant_keybind.index,
                    e.message()
                ),
            );
        }
        let next_id = (wm::messages::hotkey_identifiers::VARIANT_START
            + 2 * variant_keybind.index
            + 1) as i32;
        if let Err(e) = windows_api::unregister_hot_key(None, next_id) {
            util::add_to_message(
                warnings_string,
                &format!(
                    "Warning: Failed to unregister variant_keybinds hotkey (index: {}, next)\n{}",
                    variant_keybind.index,
                    e.message()
                ),
            );
        }
    }
}
