use crate::{windows_api, wm};
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        UI::{Accessibility::*, WindowsAndMessaging::*},
    },
};

pub extern "system" fn event_handler(
    _hwineventhook: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    idobject: i32,
    _idchild: i32,
    _ideventthread: u32,
    _dwmseventtime: u32,
) {
    if event == EVENT_OBJECT_DESTROY {
        if !wm::util::has_sizebox(hwnd) {
            return;
        }
    } else if !wm::util::is_overlapped_window(hwnd) {
        return;
    }
    match event {
        EVENT_OBJECT_SHOW if idobject == OBJID_WINDOW.0 => {
            windows_api::post_message(
                None,
                wm::messages::messages::WINDOW_CREATED,
                WPARAM(hwnd.0 as usize),
                LPARAM::default(),
            )
            .unwrap();
        }
        EVENT_OBJECT_DESTROY if idobject == OBJID_WINDOW.0 => {
            windows_api::post_message(
                None,
                wm::messages::messages::WINDOW_DESTROYED,
                WPARAM(hwnd.0 as usize),
                LPARAM::default(),
            )
            .unwrap();
        }
        EVENT_OBJECT_LOCATIONCHANGE => {
            if wm::util::is_restored(hwnd) {
                windows_api::post_message(
                    None,
                    wm::messages::messages::WINDOW_RESTORED,
                    WPARAM(hwnd.0 as usize),
                    LPARAM::default(),
                )
                .unwrap();
            } else {
                windows_api::post_message(
                    None,
                    wm::messages::messages::STOP_MANAGING_WINDOW,
                    WPARAM(hwnd.0 as usize),
                    LPARAM::default(),
                )
                .unwrap();
            }
        }
        EVENT_OBJECT_HIDE if idobject == OBJID_WINDOW.0 => {
            windows_api::post_message(
                None,
                wm::messages::messages::STOP_MANAGING_WINDOW,
                WPARAM(hwnd.0 as usize),
                LPARAM::default(),
            )
            .unwrap();
        }
        EVENT_OBJECT_CLOAKED if idobject == OBJID_WINDOW.0 => {
            windows_api::post_message(
                None,
                wm::messages::messages::WINDOW_CLOAKED,
                WPARAM(hwnd.0 as usize),
                LPARAM::default(),
            )
            .unwrap();
        }
        EVENT_OBJECT_UNCLOAKED if idobject == OBJID_WINDOW.0 => {
            windows_api::post_message(
                None,
                wm::messages::messages::WINDOW_UNCLOAKED,
                WPARAM(hwnd.0 as usize),
                LPARAM::default(),
            )
            .unwrap();
        }
        EVENT_SYSTEM_FOREGROUND | EVENT_OBJECT_FOCUS => {
            windows_api::post_message(
                None,
                wm::messages::messages::FOREGROUND_WINDOW_CHANGED,
                WPARAM(hwnd.0 as usize),
                LPARAM::default(),
            )
            .unwrap();
        }
        EVENT_SYSTEM_MOVESIZEEND => {
            windows_api::post_message(
                None,
                wm::messages::messages::WINDOW_MOVE_FINISHED,
                WPARAM(hwnd.0 as usize),
                LPARAM::default(),
            )
            .unwrap();
        }
        _ => return,
    }
}

pub unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let wm = &mut *(lparam.0 as *mut wm::WindowManager);
    let desktop_id = match windows_api::get_window_desktop_id(wm.virtual_desktop_manager(), hwnd) {
        Ok(guid) if guid != GUID::zeroed() => guid,
        _ => return true.into(),
    };
    let monitor_handle = windows_api::monitor_from_window(hwnd, MONITOR_DEFAULTTONULL);
    if monitor_handle.is_invalid()
        || !windows_api::is_window_visible(hwnd).as_bool()
        || !wm::util::is_overlapped_window(hwnd)
    {
        return true.into();
    }
    wm.manage_new_window(desktop_id, monitor_handle, hwnd);
    return true.into();
}

pub unsafe extern "system" fn enum_display_monitors_callback(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _hdc_monitor: *mut RECT,
    dw_data: LPARAM,
) -> BOOL {
    let wm = &mut *(dw_data.0 as *mut wm::WindowManager);
    wm.monitor_handles_mut().push(hmonitor);
    wm.layouts_mut().insert(hmonitor.0, Vec::new());
    return true.into();
}
