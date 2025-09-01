use crate::windows_api;
use himewm_layout::*;
use windows::{
    core::PSTR,
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::Threading::*,
        UI::WindowsAndMessaging::*,
    },
};

const MAX_PATH_LEN: usize = 1024;

pub fn is_restored(hwnd: HWND) -> bool {
    return has_sizebox(hwnd)
        && !windows_api::is_iconic(hwnd).as_bool()
        && !windows_api::is_zoomed(hwnd).as_bool()
        && !windows_api::is_window_arranged(hwnd).as_bool()
        && windows_api::is_window_visible(hwnd).as_bool();
}

pub fn has_sizebox(hwnd: HWND) -> bool {
    windows_api::get_window_long_ptr(hwnd, GWL_STYLE) & WS_SIZEBOX.0 as isize != 0
}

pub fn is_overlapped_window(hwnd: HWND) -> bool {
    windows_api::get_window_long_ptr(hwnd, GWL_STYLE) & WS_OVERLAPPEDWINDOW.0 as isize != 0
}

pub fn convert_layout_for_monitor(layout: &Layout, hmonitor: HMONITOR) -> Option<Layout> {
    let mut monitor_info = MONITORINFO::default();
    monitor_info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
    let _ = windows_api::get_monitor_info(hmonitor, &mut monitor_info);
    let monitor_rect = Zone::from(monitor_info.rcWork);
    let variant_monitor_rect = layout.get_monitor_rect();
    if &monitor_rect == variant_monitor_rect {
        return None;
    }
    let original_width = (variant_monitor_rect.right - variant_monitor_rect.left) as f64;
    let original_height = (variant_monitor_rect.bottom - variant_monitor_rect.top) as f64;
    let new_width = (monitor_rect.right - monitor_rect.left) as f64;
    let new_height = (monitor_rect.bottom - monitor_rect.top) as f64;
    let mut ret = layout.clone();
    for l in ret.get_variants_mut().iter_mut() {
        for zones in l.get_zones_mut().iter_mut() {
            for zone in zones {
                zone.left -= variant_monitor_rect.left;
                zone.top -= variant_monitor_rect.top;
                zone.right -= variant_monitor_rect.left;
                zone.bottom -= variant_monitor_rect.top;
                if new_width != original_width {
                    zone.left = ((zone.left as f64 * new_width) / original_width).round() as i32;
                    zone.right = ((zone.right as f64 * new_width) / original_width).round() as i32;
                }
                if new_height != original_height {
                    zone.top = ((zone.top as f64 * new_height) / original_height).round() as i32;
                    zone.bottom =
                        ((zone.bottom as f64 * new_height) / original_height).round() as i32;
                }
                zone.left += (&monitor_rect).left;
                zone.top += (&monitor_rect).top;
                zone.right += (&monitor_rect).left;
                zone.bottom += (&monitor_rect).top;
            }
        }
    }
    ret.set_monitor_rect(monitor_rect.clone());
    return Some(ret);
}

pub fn get_window_title(hwnd: HWND) -> Result<String, std::string::FromUtf8Error> {
    let len = windows_api::get_window_text_length(hwnd) as usize;
    let mut buf = vec![0 as u8; len + 1];
    windows_api::get_window_text(hwnd, &mut buf);
    return String::from_utf8(buf);
}

pub fn get_exe_name(hwnd: HWND) -> Result<String, std::string::FromUtf8Error> {
    let mut id = 0;
    windows_api::get_window_thread_process_id(hwnd, Some(&mut id));
    let handle = windows_api::open_process(PROCESS_QUERY_LIMITED_INFORMATION, false, id).unwrap();
    let mut buf = [0 as u8; MAX_PATH_LEN];
    let mut size = MAX_PATH_LEN as u32;
    let _query = windows_api::query_full_process_image_name(
        handle,
        PROCESS_NAME_FORMAT(0),
        PSTR(&mut buf as *mut u8),
        &mut size,
    );
    let _close_handle = windows_api::close_handle(handle);
    let path_string = String::from_utf8(Vec::from(&buf[0..size as usize]))?;
    let path = std::path::Path::new(&path_string);
    return Ok(String::from(path.file_name().unwrap().to_str().unwrap()));
}
