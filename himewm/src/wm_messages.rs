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
}

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
    pub const REFRESH_WORKSPACE: usize = 16;
    pub const RESTART_HIMEWM: usize = 17;
}
