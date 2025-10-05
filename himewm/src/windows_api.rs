use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::{Dwm::*, Gdi::*},
        System::{Com::*, Console::*, Threading::*},
        UI::{
            Accessibility::*, HiDpi::*, Input::KeyboardAndMouse::*, Shell::*,
            WindowsAndMessaging::*,
        },
    },
};

pub fn co_initialize_ex(pvreserved: Option<*const core::ffi::c_void>, dwcoinit: COINIT) -> HRESULT {
    unsafe {
        return CoInitializeEx(pvreserved, dwcoinit);
    }
}
pub fn set_win_event_hook(
    eventmin: u32,
    eventmax: u32,
    hmodwineventproc: Option<HMODULE>,
    pfnwineventproc: WINEVENTPROC,
    idprocess: u32,
    idthread: u32,
    dwflags: u32,
) -> HWINEVENTHOOK {
    unsafe {
        return SetWinEventHook(
            eventmin,
            eventmax,
            hmodwineventproc,
            pfnwineventproc,
            idprocess,
            idthread,
            dwflags,
        );
    }
}

pub fn co_create_instance<P1, T>(
    rclsid: *const GUID,
    punkouter: P1,
    dwclscontext: CLSCTX,
) -> Result<T>
where
    P1: Param<IUnknown>,
    T: Interface,
{
    unsafe {
        return CoCreateInstance(rclsid, punkouter, dwclscontext);
    }
}

pub fn enum_display_monitors(
    hdc: Option<HDC>,
    lprcclip: Option<*const RECT>,
    lpfnenum: MONITORENUMPROC,
    dwdata: LPARAM,
) -> BOOL {
    unsafe {
        return EnumDisplayMonitors(hdc, lprcclip, lpfnenum, dwdata);
    }
}

pub fn enum_windows(lpenumfunc: WNDENUMPROC, lparam: LPARAM) -> Result<()> {
    unsafe {
        return EnumWindows(lpenumfunc, lparam);
    }
}

pub fn get_foreground_window() -> HWND {
    unsafe {
        return GetForegroundWindow();
    }
}

pub fn get_window_desktop_id(
    i_virtual_desktop_manager: &IVirtualDesktopManager,
    toplevelwindow: HWND,
) -> Result<GUID> {
    unsafe {
        return i_virtual_desktop_manager.GetWindowDesktopId(toplevelwindow);
    }
}

pub fn monitor_from_window(hwnd: HWND, dwflags: MONITOR_FROM_FLAGS) -> HMONITOR {
    unsafe {
        return MonitorFromWindow(hwnd, dwflags);
    }
}

pub fn set_foreground_window(hwnd: HWND) -> BOOL {
    unsafe {
        return SetForegroundWindow(hwnd);
    }
}

pub fn show_window(hwnd: HWND, ncmdshow: SHOW_WINDOW_CMD) -> BOOL {
    unsafe {
        return ShowWindow(hwnd, ncmdshow);
    }
}

pub fn get_window_rect(hwnd: HWND, lprect: *mut RECT) -> Result<()> {
    unsafe {
        return GetWindowRect(hwnd, lprect);
    }
}

pub fn get_dpi_for_window(hwnd: HWND) -> u32 {
    unsafe {
        return GetDpiForWindow(hwnd);
    }
}

pub fn set_window_pos(
    hwnd: HWND,
    hwndinsertafter: Option<HWND>,
    x: i32,
    y: i32,
    cx: i32,
    cy: i32,
    uflags: SET_WINDOW_POS_FLAGS,
) -> Result<()> {
    unsafe {
        return SetWindowPos(hwnd, hwndinsertafter, x, y, cx, cy, uflags);
    }
}

pub fn get_last_error() -> WIN32_ERROR {
    unsafe {
        return GetLastError();
    }
}

pub fn dwm_set_window_attribute(
    hwnd: HWND,
    dwattribute: DWMWINDOWATTRIBUTE,
    pvattribute: *const core::ffi::c_void,
    cbattribute: u32,
) -> Result<()> {
    unsafe {
        return DwmSetWindowAttribute(hwnd, dwattribute, pvattribute, cbattribute);
    }
}

pub fn post_message(hwnd: Option<HWND>, msg: u32, wparam: WPARAM, lparam: LPARAM) -> Result<()> {
    unsafe {
        return PostMessageA(hwnd, msg, wparam, lparam);
    }
}

pub fn is_iconic(hwnd: HWND) -> BOOL {
    unsafe {
        return IsIconic(hwnd);
    }
}

pub fn is_zoomed(hwnd: HWND) -> BOOL {
    unsafe {
        return IsZoomed(hwnd);
    }
}

pub fn is_window_arranged(hwnd: HWND) -> BOOL {
    unsafe {
        return IsWindowArranged(hwnd);
    }
}

pub fn is_window_visible(hwnd: HWND) -> BOOL {
    unsafe {
        return IsWindowVisible(hwnd);
    }
}

pub fn get_window_long_ptr(hwnd: HWND, nindex: WINDOW_LONG_PTR_INDEX) -> isize {
    unsafe {
        return GetWindowLongPtrA(hwnd, nindex);
    }
}

pub fn get_monitor_info(hmonitor: HMONITOR, lpmi: *mut MONITORINFO) -> BOOL {
    unsafe {
        return GetMonitorInfoA(hmonitor, lpmi);
    }
}

pub fn register_hot_key(
    hwnd: Option<HWND>,
    id: i32,
    fsmodifiers: HOT_KEY_MODIFIERS,
    vk: u32,
) -> Result<()> {
    unsafe {
        return RegisterHotKey(hwnd, id, fsmodifiers, vk);
    }
}

pub fn unregister_hot_key(hwnd: Option<HWND>, id: i32) -> Result<()> {
    unsafe {
        return UnregisterHotKey(hwnd, id);
    }
}

pub fn get_window_text_length(hwnd: HWND) -> i32 {
    unsafe {
        return GetWindowTextLengthA(hwnd);
    }
}

pub fn get_window_text(hwnd: HWND, lpstring: &mut [u8]) -> i32 {
    unsafe {
        return GetWindowTextA(hwnd, lpstring);
    }
}

pub fn get_window_thread_process_id(hwnd: HWND, lpdwprocessid: Option<*mut u32>) -> u32 {
    unsafe {
        return GetWindowThreadProcessId(hwnd, lpdwprocessid);
    }
}

pub fn open_process(
    dwdesiredaccess: PROCESS_ACCESS_RIGHTS,
    binherithandle: bool,
    dwprocessid: u32,
) -> Result<HANDLE> {
    unsafe {
        return OpenProcess(dwdesiredaccess, binherithandle, dwprocessid);
    }
}

pub fn query_full_process_image_name(
    hprocess: HANDLE,
    dwflags: PROCESS_NAME_FORMAT,
    lpexename: PSTR,
    lpdwsize: *mut u32,
) -> Result<()> {
    unsafe {
        return QueryFullProcessImageNameA(hprocess, dwflags, lpexename, lpdwsize);
    }
}

pub fn close_handle(hobject: HANDLE) -> Result<()> {
    unsafe {
        return CloseHandle(hobject);
    }
}

pub fn post_quit_message(nexitcode: i32) {
    unsafe {
        return PostQuitMessage(nexitcode);
    }
}

pub fn get_message(
    lpmsg: *mut MSG,
    hwnd: Option<HWND>,
    wmsgfiltermin: u32,
    wmsgfiltermax: u32,
) -> BOOL {
    unsafe {
        return GetMessageA(lpmsg, hwnd, wmsgfiltermin, wmsgfiltermax);
    }
}

pub fn translate_message(lpmsg: *const MSG) -> BOOL {
    unsafe {
        return TranslateMessage(lpmsg);
    }
}

pub fn dispatch_message(lpmsg: *const MSG) -> LRESULT {
    unsafe {
        return DispatchMessageA(lpmsg);
    }
}

pub fn unhook_win_event(hwineventhook: HWINEVENTHOOK) -> BOOL {
    unsafe {
        return UnhookWinEvent(hwineventhook);
    }
}

pub fn co_uninitialize() {
    unsafe {
        return CoUninitialize();
    }
}

pub fn is_window(hwnd: Option<HWND>) -> BOOL {
    unsafe {
        return IsWindow(hwnd);
    }
}

pub fn get_console_window() -> HWND {
    unsafe {
        return GetConsoleWindow();
    }
}
