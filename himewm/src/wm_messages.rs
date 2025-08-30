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
