pub mod messages {
    use windows::Win32::UI::WindowsAndMessaging::WM_APP;
    pub const WINDOW_CREATED: u32 = WM_APP + 1;
    pub const WINDOW_RESTORED: u32 = WM_APP + 2;
    pub const WINDOW_DESTROYED: u32 = WM_APP + 3;
    pub const STOP_MANAGING_WINDOW: u32 = WM_APP + 4;
    pub const WINDOW_CLOAKED: u32 = WM_APP + 5;
    pub const WINDOW_UNCLOAKED: u32 = WM_APP + 6;
    pub const FOREGROUND_WINDOW_CHANGED: u32 = WM_APP + 7;
    pub const WINDOW_MOVE_FINISHED: u32 = WM_APP + 8;
    pub const REQUEST_RESTART: u32 = WM_APP + 9;
    pub const RESTART_HIMEWM: u32 = WM_APP + 10;
}

pub mod hotkey_identifiers {
    pub const HOTKEY_IDENTIFIERS_START: usize = 0;
    pub const FOCUS_PREVIOUS: usize = 0;
    pub const FOCUS_NEXT: usize = 1;
    pub const SWAP_PREVIOUS: usize = 2;
    pub const SWAP_NEXT: usize = 3;
    pub const LAYOUT_PREVIOUS: usize = 4;
    pub const LAYOUT_NEXT: usize = 5;
    pub const FOCUS_PREVIOUS_MONITOR: usize = 6;
    pub const FOCUS_NEXT_MONITOR: usize = 7;
    pub const MOVE_TO_PREVIOUS_MONITOR: usize = 8;
    pub const MOVE_TO_NEXT_MONITOR: usize = 9;
    pub const GRAB_WINDOW: usize = 10;
    pub const RELEASE_WINDOW: usize = 11;
    pub const TOGGLE_WINDOW: usize = 12;
    pub const TOGGLE_WORKSPACE: usize = 13;
    pub const REFRESH_WORKSPACE: usize = 14;
    pub const REQUEST_RESTART: usize = 15;
    pub const VARIANT_START: usize = 16;
}

pub mod tray_menu_ids {
    pub const QUIT: &str = "quit";
    pub const RESTART: &str = "restart";
}
