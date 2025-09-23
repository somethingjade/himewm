use crate::{wm, wm_messages};
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

pub fn handle_message(msg: MSG, wm: &mut wm::WindowManager) {
    match msg.message {
        wm_messages::messages::WINDOW_CREATED => {
            wm.manage_window(HWND(msg.wParam.0 as *mut core::ffi::c_void));
        }
        wm_messages::messages::WINDOW_RESTORED
            if wm
                .get_window_info_hashmap()
                .contains_key(&(msg.wParam.0 as *mut core::ffi::c_void)) =>
        {
            wm.manage_window(HWND(msg.wParam.0 as *mut core::ffi::c_void));
        }
        wm_messages::messages::WINDOW_DESTROYED => {
            wm.window_destroyed(HWND(msg.wParam.0 as *mut core::ffi::c_void));
        }
        wm_messages::messages::STOP_MANAGING_WINDOW => {
            wm.stop_managing_window(HWND(msg.wParam.0 as *mut core::ffi::c_void));
        }
        wm_messages::messages::WINDOW_CLOAKED => {
            wm.window_cloaked(HWND(msg.wParam.0 as *mut core::ffi::c_void));
        }
        wm_messages::messages::WINDOW_UNCLOAKED => {
            wm.window_uncloaked(HWND(msg.wParam.0 as *mut core::ffi::c_void));
        }
        wm_messages::messages::FOREGROUND_WINDOW_CHANGED => {
            wm.foreground_window_changed(HWND(msg.wParam.0 as *mut core::ffi::c_void), false);
        }
        wm_messages::messages::WINDOW_MOVE_FINISHED => {
            wm.window_move_finished(HWND(msg.wParam.0 as *mut core::ffi::c_void));
        }
        wm_messages::messages::REQUEST_RESTART => {
            wm.restart_himewm();
        }
        WM_HOTKEY => match msg.wParam.0 {
            wm_messages::hotkey_identifiers::FOCUS_PREVIOUS => {
                wm.cycle_focus(wm::CycleDirection::Previous);
            }
            wm_messages::hotkey_identifiers::FOCUS_NEXT => {
                wm.cycle_focus(wm::CycleDirection::Next);
            }
            wm_messages::hotkey_identifiers::SWAP_PREVIOUS => {
                wm.cycle_swap(wm::CycleDirection::Previous);
            }
            wm_messages::hotkey_identifiers::SWAP_NEXT => {
                wm.cycle_swap(wm::CycleDirection::Next);
            }
            wm_messages::hotkey_identifiers::LAYOUT_PREVIOUS => {
                wm.cycle_layout(wm::CycleDirection::Previous);
            }
            wm_messages::hotkey_identifiers::LAYOUT_NEXT => {
                wm.cycle_layout(wm::CycleDirection::Next);
            }
            wm_messages::hotkey_identifiers::FOCUS_PREVIOUS_MONITOR => {
                wm.cycle_focused_monitor(wm::CycleDirection::Previous);
            }
            wm_messages::hotkey_identifiers::FOCUS_NEXT_MONITOR => {
                wm.cycle_focused_monitor(wm::CycleDirection::Next);
            }
            wm_messages::hotkey_identifiers::MOVE_TO_PREVIOUS_MONITOR => {
                wm.cycle_assigned_monitor(wm::CycleDirection::Previous);
            }
            wm_messages::hotkey_identifiers::MOVE_TO_NEXT_MONITOR => {
                wm.cycle_assigned_monitor(wm::CycleDirection::Next);
            }
            wm_messages::hotkey_identifiers::GRAB_WINDOW => {
                wm.grab_window();
            }
            wm_messages::hotkey_identifiers::RELEASE_WINDOW => {
                wm.release_window();
            }
            wm_messages::hotkey_identifiers::TOGGLE_WINDOW => {
                wm.toggle_window();
            }
            wm_messages::hotkey_identifiers::TOGGLE_WORKSPACE => {
                wm.toggle_workspace();
            }
            wm_messages::hotkey_identifiers::REFRESH_WORKSPACE => {
                wm.refresh_workspace();
            }
            wm_messages::hotkey_identifiers::REQUEST_RESTART => {
                wm.restart_himewm();
            }
            _ => {
                let direction = if msg.wParam.0 % 2 == 0 {
                    wm::CycleDirection::Previous
                } else {
                    wm::CycleDirection::Next
                };
                let idx = match direction {
                    wm::CycleDirection::Previous => {
                        msg.wParam.0 / 2 - wm_messages::hotkey_identifiers::VARIANT_START
                    }
                    wm::CycleDirection::Next => {
                        (msg.wParam.0 - 1) / 2 - wm_messages::hotkey_identifiers::VARIANT_START
                    }
                };
                wm.cycle_variant(direction, idx);
            }
        },
        _ => (),
    }
}
