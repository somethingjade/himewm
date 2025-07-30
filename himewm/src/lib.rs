use himewm_layout::*;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::{Dwm::*, Gdi::*},
        System::{Com::*, Console::*},
        UI::{
            Accessibility::*, HiDpi::*, Input::KeyboardAndMouse::*, Shell::*,
            WindowsAndMessaging::*,
        },
    },
};

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

mod hotkey_identifiers {
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
}

enum CycleDirection {
    Previous,
    Next,
}

pub struct Settings {
    pub default_layout_idx: usize,
    pub window_padding: i32,
    pub edge_padding: i32,
    pub disable_rounding: bool,
    pub disable_unfocused_border: bool,
    pub focused_border_colour: COLORREF,
    pub floating_window_default_w_ratio: f64,
    pub floating_window_default_h_ratio: f64,
    pub new_window_retries: i32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            default_layout_idx: 0,
            window_padding: 0,
            edge_padding: 0,
            disable_rounding: false,
            disable_unfocused_border: false,
            focused_border_colour: COLORREF(0x00FFFFFF),
            floating_window_default_w_ratio: 0.5,
            floating_window_default_h_ratio: 0.5,
            new_window_retries: 10000,
        }
    }
}

impl Settings {
    fn get_unfocused_border_colour(&self) -> COLORREF {
        if self.disable_unfocused_border {
            return COLORREF(DWMWA_COLOR_NONE);
        } else {
            return COLORREF(DWMWA_COLOR_DEFAULT);
        }
    }
}

#[derive(Clone)]
struct Workspace {
    layout_idx: usize,
    variant_idx: usize,
    managed_window_handles: Vec<HWND>,
}

impl Workspace {
    unsafe fn new(hwnd: HWND, layout_idx: usize, variant_idx: usize) -> Self {
        Self {
            layout_idx,
            variant_idx,
            managed_window_handles: vec![hwnd],
        }
    }
}

#[derive(Clone)]
struct WindowInfo {
    desktop_id: GUID,
    monitor_handle: HMONITOR,
    restored: bool,
    idx: usize,
}

impl WindowInfo {
    fn new(desktop_id: GUID, monitor_handle: HMONITOR, restored: bool, idx: usize) -> Self {
        Self {
            desktop_id,
            monitor_handle,
            restored,
            idx,
        }
    }
}

struct DesktopSwitchingState {
    uncloak_count: usize,
    max_uncloak_count: usize,
}

impl Default for DesktopSwitchingState {
    fn default() -> Self {
        Self {
            uncloak_count: 0,
            max_uncloak_count: 0,
        }
    }
}

pub struct WindowManager {
    event_hook: HWINEVENTHOOK,
    virtual_desktop_manager: IVirtualDesktopManager,
    monitor_handles: Vec<HMONITOR>,
    window_info: std::collections::HashMap<*mut core::ffi::c_void, WindowInfo>,
    workspaces: std::collections::HashMap<(GUID, *mut core::ffi::c_void), Workspace>,
    layouts: std::collections::HashMap<*mut core::ffi::c_void, Vec<Layout>>,
    foreground_window: Option<HWND>,
    grabbed_window: Option<HWND>,
    ignored_combinations: std::collections::HashSet<(GUID, *mut core::ffi::c_void)>,
    ignored_windows: std::collections::HashSet<*mut core::ffi::c_void>,
    desktop_switching_state: DesktopSwitchingState,
    settings: Settings,
}

impl WindowManager {
    pub unsafe fn new(settings: Settings) -> Self {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        Self {
            event_hook: SetWinEventHook(
                EVENT_MIN,
                EVENT_MAX,
                None,
                Some(Self::event_handler),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            ),
            virtual_desktop_manager: CoCreateInstance(
                &VirtualDesktopManager,
                None,
                CLSCTX_INPROC_SERVER,
            )
            .unwrap(),
            monitor_handles: Vec::new(),
            window_info: std::collections::HashMap::new(),
            workspaces: std::collections::HashMap::new(),
            layouts: std::collections::HashMap::new(),
            foreground_window: None,
            grabbed_window: None,
            ignored_combinations: std::collections::HashSet::new(),
            ignored_windows: std::collections::HashSet::new(),
            desktop_switching_state: DesktopSwitchingState::default(),
            settings,
        }
    }

    pub unsafe fn initialize(&mut self, layouts: Vec<Layout>) {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(Self::enum_display_monitors_callback),
            LPARAM(self as *mut WindowManager as isize),
        );
        for layout in layouts {
            let monitor_rect = layout.get_monitor_rect();
            for (hmonitor, layouts) in self.layouts.iter_mut() {
                let mut layout = match convert_for_monitor(&layout, HMONITOR(*hmonitor)) {
                    Some(val) => val,
                    None => layout.clone(),
                };
                layout.update_all(
                    self.settings.window_padding,
                    self.settings.edge_padding,
                    monitor_rect,
                );
                layouts.push(layout);
            }
        }
        EnumWindows(
            Some(Self::enum_windows_callback),
            LPARAM(self as *mut WindowManager as isize),
        )
        .unwrap();
        let foreground_window = GetForegroundWindow();
        if self.window_info.contains_key(&foreground_window.0) {
            self.foreground_window = Some(foreground_window);
            self.set_border_to_focused(foreground_window);
        }
        self.update();
    }

    pub fn get_settings(&self) -> &Settings {
        &self.settings
    }

    pub fn get_settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    pub fn get_monitor_vec(&self) -> &Vec<HMONITOR> {
        &self.monitor_handles
    }

    pub fn get_event_hook(self) -> HWINEVENTHOOK {
        self.event_hook
    }

    unsafe fn manage_window(&mut self, hwnd: HWND) {
        let desktop_id;
        let monitor_handle;
        match self.window_info.get_mut(&hwnd.0) {
            Some(window_info) if window_info.restored => return,
            Some(window_info) if is_restored(hwnd) => {
                let idx = window_info.idx;
                desktop_id = window_info.desktop_id;
                monitor_handle = window_info.monitor_handle;
                match self.foreground_window {
                    Some(foreground_hwnd) if foreground_hwnd == hwnd => {
                        self.foreground_window_changed(hwnd, true)
                    }
                    _ => (),
                }
                self.window_info.get_mut(&hwnd.0).unwrap().restored = true;
                if self.ignored_windows.contains(&hwnd.0) {
                    return;
                }
                self.insert_hwnd(desktop_id, monitor_handle, idx, hwnd);
                self.update_workspace(desktop_id, monitor_handle);
            }
            None => {
                if self.ignored_windows.contains(&hwnd.0) {
                    return;
                }
                let mut count = 0;
                loop {
                    match self.virtual_desktop_manager.GetWindowDesktopId(hwnd) {
                        Ok(guid) if guid != GUID::zeroed() => {
                            desktop_id = guid;
                            break;
                        }
                        _ => {
                            count += 1;
                        }
                    }
                    if count == self.settings.new_window_retries {
                        return;
                    }
                }
                monitor_handle = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONULL);
                if monitor_handle.is_invalid() {
                    return;
                }
                self.window_info.insert(
                    hwnd.0,
                    WindowInfo::new(desktop_id, monitor_handle, is_restored(hwnd), 0),
                );
                self.push_hwnd(desktop_id, monitor_handle, hwnd);
                self.initialize_border(hwnd);
                if let None = self.foreground_window {
                    self.foreground_window_changed(hwnd, false);
                }
            }
            _ => return,
        }
        self.update_workspace(desktop_id, monitor_handle);
    }

    unsafe fn window_destroyed(&mut self, hwnd: HWND) {
        let window_info = match self.window_info.get(&hwnd.0) {
            Some(val) => val,
            None => {
                self.ignored_windows.remove(&hwnd.0);
                return;
            }
        };
        let WindowInfo {
            desktop_id,
            monitor_handle,
            restored,
            idx,
        } = window_info.to_owned();
        self.window_info.remove(&hwnd.0);
        if self.foreground_window == Some(hwnd) {
            self.foreground_window = None;
        }
        if self.grabbed_window == Some(hwnd) {
            self.grabbed_window = None;
        }
        if restored && !self.ignored_windows.contains(&hwnd.0) {
            self.remove_hwnd(desktop_id, monitor_handle, idx);
            self.update_workspace(desktop_id, monitor_handle);
        }
    }

    unsafe fn stop_managing_window(&mut self, hwnd: HWND) {
        let window_info = match self.window_info.get_mut(&hwnd.0) {
            Some(val) if val.restored => val,
            _ => return,
        };
        let WindowInfo {
            desktop_id,
            monitor_handle,
            restored: _,
            idx,
        } = window_info.to_owned();
        window_info.restored = false;
        if self.grabbed_window == Some(hwnd) && !self.ignored_windows.contains(&hwnd.0) {
            self.grabbed_window = None;
        }
        if !self.ignored_windows.contains(&hwnd.0) {
            self.remove_hwnd(desktop_id, monitor_handle, idx);
            self.update_workspace(desktop_id, monitor_handle);
        }
    }

    unsafe fn window_cloaked(&mut self, hwnd: HWND) {
        let window_info = match self.window_info.get(&hwnd.0) {
            Some(val) => val,
            None => return,
        };
        let WindowInfo {
            desktop_id: old_desktop_id,
            monitor_handle,
            restored,
            idx: old_idx,
        } = window_info.to_owned();
        let new_desktop_id = match self.virtual_desktop_manager.GetWindowDesktopId(hwnd) {
            Ok(guid) if guid != old_desktop_id => guid,
            _ => return,
        };
        if restored && !self.ignored_windows.contains(&hwnd.0) {
            self.remove_hwnd(old_desktop_id, monitor_handle, old_idx);
            self.push_hwnd(new_desktop_id, monitor_handle, hwnd);
        } else {
            let new_idx = match self.workspaces.get(&(new_desktop_id, monitor_handle.0)) {
                Some(workspace) => workspace.managed_window_handles.len(),
                None => 0,
            };
            self.window_info.insert(
                hwnd.0,
                WindowInfo::new(new_desktop_id, monitor_handle, restored, new_idx),
            );
        }
        self.update_workspace(old_desktop_id, monitor_handle);
        self.update_workspace(new_desktop_id, monitor_handle);
    }

    unsafe fn window_uncloaked(&mut self, hwnd: HWND) {
        let uncloaked_desktop_id = match self.window_info.get(&hwnd.0) {
            Some(val) if val.restored => val.desktop_id,
            _ => return,
        };
        if self.desktop_switching_state.uncloak_count
            == self.desktop_switching_state.max_uncloak_count
        {
            self.desktop_switching_state.uncloak_count = 0;
            self.desktop_switching_state.max_uncloak_count = 0;
        }
        if self.desktop_switching_state.uncloak_count == 0 {
            for monitor_handle in self.monitor_handles.to_owned() {
                match self
                    .workspaces
                    .get(&(uncloaked_desktop_id, monitor_handle.0))
                {
                    Some(workspace) => {
                        self.desktop_switching_state.max_uncloak_count +=
                            workspace.managed_window_handles.len();
                    }
                    None => (),
                }
            }
            self.desktop_switching_state.uncloak_count += 1;
            let foreground_hwnd = match self.foreground_window {
                Some(h) if h != hwnd => h,
                _ => {
                    return;
                }
            };
            let previous_desktop_id = self.window_info.get(&foreground_hwnd.0).unwrap().desktop_id;
            let mut new_desktop_id = None;
            let gathered_hwnds = self
                .window_info
                .iter()
                .filter_map(|(h, info)| {
                    if info.desktop_id == previous_desktop_id {
                        Some(*h)
                    } else {
                        None
                    }
                })
                .collect::<Vec<*mut core::ffi::c_void>>();
            for h in gathered_hwnds {
                let info = self.window_info.get(&h).unwrap().to_owned();
                match self.virtual_desktop_manager.GetWindowDesktopId(HWND(h)) {
                    Ok(guid) if guid != previous_desktop_id => {
                        if info.restored && !self.ignored_windows.contains(&h) {
                            self.remove_hwnd(previous_desktop_id, info.monitor_handle, info.idx);
                            self.push_hwnd(guid, info.monitor_handle, HWND(h));
                        } else {
                            let new_idx = match self.workspaces.get(&(guid, info.monitor_handle.0))
                            {
                                Some(workspace) => workspace.managed_window_handles.len(),
                                None => 0,
                            };
                            self.window_info.insert(
                                h,
                                WindowInfo::new(guid, info.monitor_handle, info.restored, new_idx),
                            );
                        }
                        new_desktop_id = Some(guid);
                    }
                    _ => (),
                }
            }
            if let Some(guid) = new_desktop_id {
                for monitor_handle in self.monitor_handles.to_owned() {
                    self.update_workspace(previous_desktop_id, monitor_handle);
                    self.update_workspace(guid, monitor_handle);
                }
            }
        } else {
            self.desktop_switching_state.uncloak_count += 1;
        }
    }

    unsafe fn foreground_window_changed(&mut self, hwnd: HWND, updating: bool) {
        if !self.window_info.contains_key(&hwnd.0) {
            if let Some(previous_foreground_window) = self.foreground_window {
                self.set_border_to_unfocused(previous_foreground_window);
            }
            self.foreground_window = None;
            return;
        }
        let window_info = self.window_info.get(&hwnd.0).unwrap();
        let WindowInfo {
            desktop_id,
            monitor_handle,
            ..
        } = window_info.to_owned();
        self.set_border_to_focused(hwnd);
        match self.foreground_window {
            Some(previous_foreground_window) if previous_foreground_window == hwnd => {
                if !updating {
                    return;
                }
            }
            Some(previous_foreground_window) => {
                self.set_border_to_unfocused(previous_foreground_window);
            }
            None => (),
        }
        self.foreground_window = Some(hwnd);
        if self
            .ignored_combinations
            .contains(&(desktop_id, monitor_handle.0))
        {
            return;
        }
        if !self.ignored_windows.contains(&hwnd.0) && is_restored(hwnd) {
            for (h, info) in self.window_info.iter_mut() {
                if h != &hwnd.0
                    && info.desktop_id == desktop_id
                    && info.monitor_handle == monitor_handle
                    && (!info.restored || self.ignored_windows.contains(h))
                {
                    let _ = ShowWindow(HWND(*h), SW_MINIMIZE);
                }
            }
        }
    }

    unsafe fn window_move_finished(&mut self, hwnd: HWND) {
        if self.ignored_windows.contains(&hwnd.0) {
            return;
        }
        let window_info = match self.window_info.get_mut(&hwnd.0) {
            Some(val) if val.restored => val,
            _ => return,
        };
        let WindowInfo {
            desktop_id,
            monitor_handle: original_monitor_handle,
            restored,
            idx,
        } = window_info.to_owned();
        let new_monitor_handle = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONULL);
        if !restored {
            window_info.monitor_handle = new_monitor_handle;
            window_info.idx = match self.workspaces.get(&(desktop_id, new_monitor_handle.0)) {
                Some(w) => w.managed_window_handles.len(),
                None => 0,
            };
            return;
        }
        let changed_monitors = original_monitor_handle != new_monitor_handle;
        let mut moved_to = RECT::default();
        GetWindowRect(hwnd, &mut moved_to).unwrap();
        let moved_to_area = (moved_to.right - moved_to.left) * (moved_to.bottom - moved_to.top);
        let workspace;
        if changed_monitors {
            workspace = match self.workspaces.get_mut(&(desktop_id, new_monitor_handle.0)) {
                Some(w) => w,
                None => {
                    self.workspaces
                        .get_mut(&(desktop_id, original_monitor_handle.0))
                        .unwrap()
                        .managed_window_handles
                        .remove(idx);
                    self.workspaces.insert(
                        (desktop_id, new_monitor_handle.0),
                        Workspace::new(
                            hwnd,
                            self.settings.default_layout_idx,
                            self.layouts.get(&new_monitor_handle.0).unwrap()
                                [self.settings.default_layout_idx]
                                .default_variant_idx(),
                        ),
                    );
                    window_info.monitor_handle = new_monitor_handle;
                    window_info.idx = 0;
                    self.update_workspace(desktop_id, original_monitor_handle);
                    self.update_workspace(desktop_id, new_monitor_handle);
                    return;
                }
            }
        } else {
            workspace = match self
                .workspaces
                .get_mut(&(desktop_id, original_monitor_handle.0))
            {
                Some(w) => w,
                None => return,
            };
        }
        let mut max_overlap_at: (usize, i32) = (workspace.managed_window_handles.len(), 0);
        {
            let positions = if changed_monitors {
                let layout =
                    &mut self.layouts.get_mut(&new_monitor_handle.0).unwrap()[workspace.layout_idx];
                let monitor_rect = layout.get_monitor_rect().to_owned();
                let variant = &mut layout.get_variants_mut()[workspace.variant_idx];
                while variant.positions_len() < workspace.managed_window_handles.len() {
                    variant.extend();
                    variant.update(
                        self.settings.window_padding,
                        self.settings.edge_padding,
                        &monitor_rect,
                    );
                }
                variant.get_positions_at(workspace.managed_window_handles.len())
            } else {
                self.layouts.get(&original_monitor_handle.0).unwrap()[workspace.layout_idx]
                    .get_variants()[workspace.variant_idx]
                    .get_positions_at(workspace.managed_window_handles.len() - 1)
            };
            if !changed_monitors {
                let position = &positions[idx];
                if moved_to.left == position.x
                    && moved_to.top == position.y
                    && moved_to.right - moved_to.left == position.cx
                    && moved_to.bottom - moved_to.top == position.cy
                {
                    return;
                }
            }
            for (i, p) in positions.iter().enumerate() {
                let left = std::cmp::max(moved_to.left, p.x);
                let top = std::cmp::max(moved_to.top, p.y);
                let right = std::cmp::min(moved_to.right, p.x + p.cx);
                let bottom = std::cmp::min(moved_to.bottom, p.y + p.cy);
                let area = (right - left) * (bottom - top);
                if area == moved_to_area {
                    max_overlap_at = (i, area);
                    break;
                } else if area > max_overlap_at.1 {
                    max_overlap_at = (i, area);
                }
            }
        }
        if changed_monitors {
            self.move_windows_across_monitors(
                desktop_id,
                original_monitor_handle,
                new_monitor_handle,
                idx,
                max_overlap_at.0,
            );
            self.update_workspace(desktop_id, original_monitor_handle);
            self.update_workspace(desktop_id, new_monitor_handle);
        } else {
            if idx != max_overlap_at.0 {
                self.swap_windows(desktop_id, original_monitor_handle, idx, max_overlap_at.0);
            }
            self.update_workspace(desktop_id, original_monitor_handle);
        }
    }

    unsafe fn cycle_focus(&self, direction: CycleDirection) {
        let foreground_window = match self.foreground_window {
            Some(hwnd) if !self.ignored_windows.contains(&hwnd.0) => hwnd,
            _ => return,
        };
        let window_info = match self.window_info.get(&foreground_window.0) {
            Some(val) if val.restored => val,
            _ => return,
        };
        let WindowInfo {
            desktop_id,
            monitor_handle,
            restored: _,
            idx,
        } = window_info.to_owned();
        let workspace = match self.workspaces.get(&(desktop_id, monitor_handle.0)) {
            Some(val) if val.managed_window_handles.len() > 1 => val,
            _ => return,
        };
        let to = match direction {
            CycleDirection::Previous => {
                if idx == 0 {
                    workspace.managed_window_handles.len() - 1
                } else {
                    idx - 1
                }
            }
            CycleDirection::Next => {
                if idx == workspace.managed_window_handles.len() - 1 {
                    0
                } else {
                    idx + 1
                }
            }
        };
        let _ = SetForegroundWindow(workspace.managed_window_handles[to]);
    }

    unsafe fn cycle_swap(&mut self, direction: CycleDirection) {
        let foreground_window = match self.foreground_window {
            Some(hwnd) => hwnd,
            None => return,
        };
        let window_info = match self.window_info.get(&foreground_window.0) {
            Some(val) if val.restored => val,
            _ => return,
        };
        let WindowInfo {
            desktop_id,
            monitor_handle,
            restored: _,
            idx,
        } = window_info.to_owned();
        if self
            .ignored_combinations
            .contains(&(desktop_id, monitor_handle.0))
        {
            return;
        }
        let workspace = match self.workspaces.get(&(desktop_id, monitor_handle.0)) {
            Some(val) if val.managed_window_handles.len() > 1 => val,
            _ => return,
        };
        let swap_with = match direction {
            CycleDirection::Previous => {
                if idx == 0 {
                    workspace.managed_window_handles.len() - 1
                } else {
                    idx - 1
                }
            }
            CycleDirection::Next => {
                if idx == workspace.managed_window_handles.len() - 1 {
                    0
                } else {
                    idx + 1
                }
            }
        };
        self.swap_windows(desktop_id, monitor_handle, idx, swap_with);
        self.update_workspace(desktop_id, monitor_handle);
    }

    unsafe fn cycle_variant(&mut self, direction: CycleDirection) {
        let foreground_window = match self.foreground_window {
            Some(hwnd) => hwnd,
            None => return,
        };
        let window_info = match self.window_info.get(&foreground_window.0) {
            Some(val) if val.restored => val,
            _ => return,
        };
        let WindowInfo {
            desktop_id,
            monitor_handle,
            ..
        } = window_info.to_owned();
        if self
            .ignored_combinations
            .contains(&(desktop_id, monitor_handle.0))
        {
            return;
        }
        let workspace = match self.workspaces.get_mut(&(desktop_id, monitor_handle.0)) {
            Some(val) => val,
            _ => return,
        };
        let variants_len =
            self.layouts.get(&monitor_handle.0).unwrap()[workspace.layout_idx].variants_len();
        if variants_len == 1 {
            return;
        }
        match direction {
            CycleDirection::Previous => {
                if workspace.variant_idx != 0 {
                    workspace.variant_idx -= 1;
                } else {
                    workspace.variant_idx = variants_len - 1;
                }
            }
            CycleDirection::Next => {
                if workspace.variant_idx != variants_len - 1 {
                    workspace.variant_idx += 1;
                } else {
                    workspace.variant_idx = 0;
                }
            }
        }
        self.update_workspace(desktop_id, monitor_handle);
    }

    unsafe fn cycle_layout(&mut self, direction: CycleDirection) {
        let foreground_window = match self.foreground_window {
            Some(hwnd) => hwnd,
            None => return,
        };
        let window_info = match self.window_info.get(&foreground_window.0) {
            Some(val) if val.restored => val,
            _ => return,
        };
        let WindowInfo {
            desktop_id,
            monitor_handle,
            ..
        } = window_info.to_owned();
        if self
            .ignored_combinations
            .contains(&(desktop_id, monitor_handle.0))
        {
            return;
        }
        let workspace = match self.workspaces.get_mut(&(desktop_id, monitor_handle.0)) {
            Some(val) => val,
            _ => return,
        };
        let layouts = self.layouts.get(&monitor_handle.0).unwrap();
        if layouts.len() == 1 {
            return;
        }
        match direction {
            CycleDirection::Previous => {
                if workspace.layout_idx == 0 {
                    workspace.layout_idx = layouts.len() - 1;
                } else {
                    workspace.layout_idx -= 1;
                }
            }
            CycleDirection::Next => {
                if workspace.layout_idx == layouts.len() - 1 {
                    workspace.layout_idx = 0;
                } else {
                    workspace.layout_idx += 1;
                }
            }
        }
        workspace.variant_idx = layouts[workspace.layout_idx].default_variant_idx();
        self.update_workspace(desktop_id, monitor_handle);
    }

    unsafe fn cycle_focused_monitor(&self, direction: CycleDirection) {
        if self.monitor_handles.len() <= 1 {
            return;
        }
        let foreground_window = match self.foreground_window {
            Some(hwnd) => hwnd,
            None => return,
        };
        let window_info = match self.window_info.get(&foreground_window.0) {
            Some(val) if val.restored => val,
            _ => return,
        };
        let WindowInfo {
            desktop_id,
            monitor_handle,
            ..
        } = window_info.to_owned();
        let mut idx = self.monitor_handles.len();
        for i in 0..self.monitor_handles.len() {
            if self.monitor_handles[i] == monitor_handle {
                idx = i;
            }
        }
        if idx == self.monitor_handles.len() {
            return;
        }
        match direction {
            CycleDirection::Previous => {
                if idx == 0 {
                    idx = self.monitor_handles.len() - 1;
                } else {
                    idx -= 1;
                }
            }
            CycleDirection::Next => {
                if idx == self.monitor_handles.len() - 1 {
                    idx = 0;
                } else {
                    idx += 1;
                }
            }
        }
        let workspace = match self
            .workspaces
            .get(&(desktop_id, self.monitor_handles[idx].0))
        {
            Some(val) if val.managed_window_handles.len() != 0 => val,
            _ => return,
        };
        let _ = SetForegroundWindow(workspace.managed_window_handles[0]);
    }

    unsafe fn cycle_assigned_monitor(&mut self, direction: CycleDirection) {
        if self.monitor_handles.len() <= 1 {
            return;
        }
        let foreground_window = match self.foreground_window {
            Some(hwnd) if !self.ignored_windows.contains(&hwnd.0) => hwnd,
            _ => return,
        };
        let original_dpi = GetDpiForWindow(foreground_window);
        let window_info = match self.window_info.get(&foreground_window.0) {
            Some(val) if val.restored => val,
            _ => return,
        };
        let WindowInfo {
            desktop_id,
            monitor_handle: original_monitor_handle,
            restored: _,
            idx: original_window_idx,
        } = window_info.to_owned();
        if self
            .ignored_combinations
            .contains(&(desktop_id, original_monitor_handle.0))
        {
            return;
        }
        let mut monitor_handle_idx = self.monitor_handles.len();
        for i in 0..self.monitor_handles.len() {
            if self.monitor_handles[i] == original_monitor_handle {
                monitor_handle_idx = i;
            }
        }
        let mut new_monitor_handle = HMONITOR::default();
        if monitor_handle_idx == self.monitor_handles.len() {
            return;
        }
        match direction {
            CycleDirection::Previous => {
                for i in 0..self.monitor_handles.len() {
                    if i == self.monitor_handles.len() - 1 {
                        return;
                    }
                    if monitor_handle_idx == 0 {
                        monitor_handle_idx = self.monitor_handles.len() - 1;
                    } else {
                        monitor_handle_idx -= 1;
                    }
                    new_monitor_handle = self.monitor_handles[monitor_handle_idx];
                    if !self
                        .ignored_combinations
                        .contains(&(desktop_id, new_monitor_handle.0))
                    {
                        break;
                    }
                }
            }
            CycleDirection::Next => {
                for i in 0..self.monitor_handles.len() {
                    if i == self.monitor_handles.len() - 1 {
                        return;
                    }
                    if monitor_handle_idx == self.monitor_handles.len() - 1 {
                        monitor_handle_idx = 0;
                    } else {
                        monitor_handle_idx += 1;
                    }
                    new_monitor_handle = self.monitor_handles[monitor_handle_idx];
                    if !self
                        .ignored_combinations
                        .contains(&(desktop_id, new_monitor_handle.0))
                    {
                        break;
                    }
                }
            }
        }
        match self.workspaces.get(&(desktop_id, new_monitor_handle.0)) {
            Some(w) => {
                self.move_windows_across_monitors(
                    desktop_id,
                    original_monitor_handle,
                    new_monitor_handle,
                    original_window_idx,
                    w.managed_window_handles.len(),
                );
            }
            None => {
                self.remove_hwnd(desktop_id, original_monitor_handle, original_window_idx);
                self.workspaces.insert(
                    (desktop_id, new_monitor_handle.0),
                    Workspace::new(
                        foreground_window,
                        self.settings.default_layout_idx,
                        self.layouts.get(&new_monitor_handle.0).unwrap()
                            [self.settings.default_layout_idx]
                            .default_variant_idx(),
                    ),
                );
                let window_info_mut = self.window_info.get_mut(&foreground_window.0).unwrap();
                window_info_mut.monitor_handle = new_monitor_handle;
                window_info_mut.idx = 0;
            }
        };
        self.update_workspace(desktop_id, original_monitor_handle);
        self.update_workspace(desktop_id, new_monitor_handle);
        if GetDpiForWindow(foreground_window) != original_dpi {
            let workspace = self
                .workspaces
                .get(&(desktop_id, new_monitor_handle.0))
                .unwrap();
            let layout = &self.layouts.get(&new_monitor_handle.0).unwrap()[workspace.layout_idx]
                .get_variants()[workspace.variant_idx];
            let position = &layout.get_positions_at(workspace.managed_window_handles.len() - 1)
                [workspace.managed_window_handles.len() - 1];
            let _ = SetWindowPos(
                foreground_window,
                None,
                position.x,
                position.y,
                position.cx,
                position.cy,
                SWP_NOZORDER,
            );
        }
    }

    pub fn grab_window(&mut self) {
        self.grabbed_window = match self.foreground_window {
            Some(hwnd) => match self.window_info.get(&hwnd.0) {
                Some(val) if val.restored => Some(hwnd),
                _ => None,
            },
            None => None,
        }
    }

    unsafe fn release_window(&mut self) {
        let grabbed_window = match self.grabbed_window {
            Some(hwnd) => hwnd,
            None => return,
        };
        let foreground_window = match self.foreground_window {
            Some(hwnd) if hwnd != grabbed_window => hwnd,
            _ => return,
        };
        let new_window_info = match self.window_info.get(&foreground_window.0) {
            Some(val) if val.restored => val,
            _ => return,
        };
        let WindowInfo {
            desktop_id: new_desktop_id,
            monitor_handle: new_monitor_handle,
            restored: _,
            idx: new_idx,
        } = new_window_info.to_owned();
        if self
            .ignored_combinations
            .contains(&(new_desktop_id, new_monitor_handle.0))
        {
            return;
        }
        let WindowInfo {
            desktop_id: original_desktop_id,
            monitor_handle: original_monitor_handle,
            restored: _,
            idx: original_idx,
        } = self
            .window_info
            .get(&self.grabbed_window.unwrap().0)
            .unwrap()
            .to_owned();
        if original_desktop_id != new_desktop_id {
            return;
        }
        let was_ignored = if self.ignored_windows.remove(&grabbed_window.0) {
            let original_window_info = self
                .window_info
                .get_mut(&self.grabbed_window.unwrap().0)
                .unwrap();
            original_window_info.restored = true;
            let _ = ShowWindow(grabbed_window, SW_RESTORE);
            true
        } else {
            false
        };
        if original_monitor_handle == new_monitor_handle {
            if was_ignored {
                self.insert_hwnd(
                    original_desktop_id,
                    original_monitor_handle,
                    original_idx,
                    grabbed_window,
                );
            }
            self.swap_windows(
                original_desktop_id,
                original_monitor_handle,
                original_idx,
                new_idx,
            );
            self.update_workspace(original_desktop_id, original_monitor_handle);
        } else {
            if was_ignored {
                self.insert_hwnd(
                    original_desktop_id,
                    new_monitor_handle,
                    new_idx,
                    grabbed_window,
                );
            } else {
                self.move_windows_across_monitors(
                    original_desktop_id,
                    original_monitor_handle,
                    new_monitor_handle,
                    original_idx,
                    new_idx,
                );
            }
            let original_dpi = GetDpiForWindow(grabbed_window);
            self.update_workspace(original_desktop_id, original_monitor_handle);
            self.update_workspace(original_desktop_id, new_monitor_handle);
            if GetDpiForWindow(grabbed_window) != original_dpi {
                let workspace = self
                    .workspaces
                    .get(&(original_desktop_id, new_monitor_handle.0))
                    .unwrap();
                let layout = &self.layouts.get(&new_monitor_handle.0).unwrap()
                    [workspace.layout_idx]
                    .get_variants()[workspace.variant_idx];
                let position =
                    &layout.get_positions_at(workspace.managed_window_handles.len() - 1)[new_idx];
                let _ = SetWindowPos(
                    grabbed_window,
                    None,
                    position.x,
                    position.y,
                    position.cx,
                    position.cy,
                    SWP_NOZORDER,
                );
            }
        }
        let _ = SetForegroundWindow(grabbed_window);
        self.grabbed_window = None;
    }

    unsafe fn toggle_window(&mut self) {
        let foreground_window = match self.foreground_window {
            Some(hwnd) => hwnd,
            None => return,
        };
        let WindowInfo {
            desktop_id,
            monitor_handle,
            restored,
            idx,
        } = match self.window_info.get(&foreground_window.0) {
            Some(val) => val.to_owned(),
            None => return,
        };
        if self.ignored_windows.remove(&foreground_window.0) {
            if restored {
                let original_dpi = GetDpiForWindow(foreground_window);
                self.foreground_window_changed(foreground_window, true);
                self.insert_hwnd(desktop_id, monitor_handle, idx, foreground_window);
                self.update_workspace(desktop_id, monitor_handle);
                if GetDpiForWindow(foreground_window) != original_dpi {
                    let workspace = self
                        .workspaces
                        .get(&(desktop_id, monitor_handle.0))
                        .unwrap();
                    let layout = &self.layouts.get(&monitor_handle.0).unwrap()
                        [workspace.layout_idx]
                        .get_variants()[workspace.variant_idx];
                    let position = &layout
                        .get_positions_at(workspace.managed_window_handles.len() - 1)
                        [workspace.managed_window_handles.len() - 1];
                    let _ = SetWindowPos(
                        foreground_window,
                        None,
                        position.x,
                        position.y,
                        position.cx,
                        position.cy,
                        SWP_NOZORDER,
                    );
                }
            }
        } else {
            self.ignored_windows.insert(foreground_window.0);
            if let None = self.remove_hwnd(desktop_id, monitor_handle, idx) {
                return;
            }
            self.update_workspace(desktop_id, monitor_handle);
            let workspace = self
                .workspaces
                .get(&(desktop_id, monitor_handle.0))
                .unwrap();
            let monitor_rect = self.layouts.get_mut(&monitor_handle.0).unwrap()
                [workspace.layout_idx]
                .get_monitor_rect();
            let w = ((monitor_rect.w() as f64) * self.settings.floating_window_default_w_ratio)
                .round() as i32;
            let h = ((monitor_rect.h() as f64) * self.settings.floating_window_default_h_ratio)
                .round() as i32;
            let x = (((monitor_rect.w() - w) as f64) * 0.5).round() as i32 + monitor_rect.left;
            let y = (((monitor_rect.h() - h) as f64) * 0.5).round() as i32 + monitor_rect.top;
            let _ = SetWindowPos(foreground_window, None, x, y, w, h, SWP_NOZORDER);
        }
    }

    unsafe fn toggle_workspace(&mut self) {
        let foreground_window = match self.foreground_window {
            Some(hwnd) => hwnd,
            None => return,
        };
        let WindowInfo {
            desktop_id,
            monitor_handle,
            ..
        } = match self.window_info.get(&foreground_window.0) {
            Some(val) => val,
            None => return,
        };
        if self
            .ignored_combinations
            .remove(&(*desktop_id, monitor_handle.0))
        {
            self.update_workspace(*desktop_id, *monitor_handle);
        } else {
            self.ignored_combinations
                .insert((*desktop_id, monitor_handle.0));
        }
    }

    unsafe fn update_workspace(&mut self, guid: GUID, hmonitor: HMONITOR) {
        if self.ignored_combinations.contains(&(guid, hmonitor.0)) {
            return;
        }
        let workspace = match self.workspaces.get(&(guid, hmonitor.0)) {
            Some(w) => w,
            None => return,
        };
        if workspace.managed_window_handles.len() == 0 {
            return;
        }
        let layout = &mut self.layouts.get_mut(&hmonitor.0).unwrap()[workspace.layout_idx];
        let monitor_rect = layout.get_monitor_rect().to_owned();
        let variant = &mut layout.get_variants_mut()[workspace.variant_idx];
        while variant.positions_len() < workspace.managed_window_handles.len() {
            variant.extend();
            variant.update(
                self.settings.window_padding,
                self.settings.edge_padding,
                &monitor_rect,
            );
        }
        let mut error_indices: Option<Vec<usize>> = None;
        let positions = variant.get_positions_at(workspace.managed_window_handles.len() - 1);
        for (i, hwnd) in workspace.managed_window_handles.iter().enumerate() {
            match SetWindowPos(
                *hwnd,
                None,
                positions[i].x,
                positions[i].y,
                positions[i].cx,
                positions[i].cy,
                SWP_NOZORDER,
            ) {
                Ok(_) => continue,
                Err(_) => {
                    match &mut error_indices {
                        Some(v) => v.push(i),
                        None => {
                            error_indices = Some(vec![i]);
                        }
                    }
                    self.window_info.remove(&hwnd.0);
                    if GetLastError().0 == 5 {
                        self.ignored_windows.insert(hwnd.0);
                    }
                }
            }
        }
        if let Some(v) = error_indices {
            for (i, error_idx) in v.iter().enumerate() {
                self.remove_hwnd(guid, hmonitor, *error_idx - i);
            }
            self.update_workspace(guid, hmonitor);
        }
    }

    unsafe fn update(&mut self) {
        let keys: Vec<(GUID, *mut core::ffi::c_void)> =
            self.workspaces.keys().map(|k| (k.0, k.1)).collect();
        for k in keys.iter() {
            self.update_workspace(k.0, HMONITOR(k.1));
        }
    }

    fn swap_windows(&mut self, guid: GUID, hmonitor: HMONITOR, i: usize, j: usize) {
        if i == j {
            return;
        }
        let managed_window_handles = &mut self
            .workspaces
            .get_mut(&(guid, hmonitor.0))
            .unwrap()
            .managed_window_handles;
        self.window_info
            .get_mut(&managed_window_handles[i].0)
            .unwrap()
            .idx = j;
        self.window_info
            .get_mut(&managed_window_handles[j].0)
            .unwrap()
            .idx = i;
        managed_window_handles.swap(i, j);
    }

    unsafe fn move_windows_across_monitors(
        &mut self,
        guid: GUID,
        first_hmonitor: HMONITOR,
        second_hmonitor: HMONITOR,
        first_idx: usize,
        second_idx: usize,
    ) {
        let hwnd = match self.remove_hwnd(guid, first_hmonitor, first_idx) {
            Some(val) => val,
            None => return,
        };
        self.insert_hwnd(guid, second_hmonitor, second_idx, hwnd);
    }

    unsafe fn set_border_to_unfocused(&self, hwnd: HWND) {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &self.settings.get_unfocused_border_colour() as *const COLORREF
                as *const core::ffi::c_void,
            std::mem::size_of_val(&self.settings.get_unfocused_border_colour()) as u32,
        );
    }

    unsafe fn set_border_to_focused(&self, hwnd: HWND) {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &self.settings.focused_border_colour as *const COLORREF as *const core::ffi::c_void,
            std::mem::size_of_val(&self.settings.focused_border_colour) as u32,
        );
    }

    unsafe fn initialize_border(&self, hwnd: HWND) {
        let corner_preference = if self.settings.disable_rounding {
            DWMWCP_DONOTROUND
        } else {
            DWMWCP_DEFAULT
        };
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &corner_preference as *const DWM_WINDOW_CORNER_PREFERENCE as *const core::ffi::c_void,
            std::mem::size_of_val(&corner_preference) as u32,
        );
        self.set_border_to_unfocused(hwnd);
    }

    unsafe fn insert_hwnd(&mut self, guid: GUID, hmonitor: HMONITOR, idx: usize, hwnd: HWND) {
        let window_info = match self.window_info.get_mut(&hwnd.0) {
            Some(val) => val,
            None => return,
        };
        match self.workspaces.get_mut(&(guid, hmonitor.0)) {
            Some(workspace) => {
                if window_info.restored {
                    workspace.managed_window_handles.insert(idx, hwnd);
                }
                window_info.idx = idx;
            }
            None => {
                if window_info.restored {
                    self.workspaces.insert(
                        (guid, hmonitor.0),
                        Workspace::new(
                            hwnd,
                            self.settings.default_layout_idx,
                            self.layouts.get(&window_info.monitor_handle.0).unwrap()
                                [self.settings.default_layout_idx]
                                .default_variant_idx(),
                        ),
                    );
                }
                window_info.idx = 0;
            }
        };
        window_info.desktop_id = guid;
        window_info.monitor_handle = hmonitor;
        if window_info.restored {
            for (h, info) in self.window_info.iter_mut() {
                if info.desktop_id == guid
                    && info.monitor_handle == hmonitor
                    && info.idx >= idx
                    && *h != hwnd.0
                {
                    info.idx += 1;
                }
            }
        }
    }

    unsafe fn push_hwnd(&mut self, guid: GUID, hmonitor: HMONITOR, hwnd: HWND) {
        let idx = if let Some(workspace) = self.workspaces.get(&(guid, hmonitor.0)) {
            workspace.managed_window_handles.len()
        } else {
            0
        };
        self.insert_hwnd(guid, hmonitor, idx, hwnd);
    }

    fn remove_hwnd(&mut self, guid: GUID, hmonitor: HMONITOR, idx: usize) -> Option<HWND> {
        let workspace = match self.workspaces.get_mut(&(guid, hmonitor.0)) {
            Some(val) => val,
            None => return None,
        };
        let hwnd = workspace.managed_window_handles.remove(idx);
        for info in self.window_info.values_mut() {
            if info.desktop_id == guid && info.monitor_handle == hmonitor && info.idx > idx {
                info.idx -= 1;
            }
        }
        return Some(hwnd);
    }

    unsafe extern "system" fn event_handler(
        _hwineventhook: HWINEVENTHOOK,
        event: u32,
        hwnd: HWND,
        idobject: i32,
        _idchild: i32,
        _ideventthread: u32,
        _dwmseventtime: u32,
    ) {
        if event == EVENT_OBJECT_LOCATIONCHANGE {
            if !is_overlappedwindow(hwnd) {
                return;
            }
        } else if !has_sizebox(hwnd) {
            return;
        }
        match event {
            EVENT_OBJECT_SHOW if idobject == OBJID_WINDOW.0 => {
                PostMessageA(
                    None,
                    messages::WINDOW_CREATED,
                    WPARAM(hwnd.0 as usize),
                    LPARAM::default(),
                )
                .unwrap();
            }
            EVENT_OBJECT_DESTROY if idobject == OBJID_WINDOW.0 => {
                PostMessageA(
                    None,
                    messages::WINDOW_DESTROYED,
                    WPARAM(hwnd.0 as usize),
                    LPARAM::default(),
                )
                .unwrap();
            }
            EVENT_OBJECT_LOCATIONCHANGE => {
                if is_restored(hwnd) {
                    PostMessageA(
                        None,
                        messages::WINDOW_RESTORED,
                        WPARAM(hwnd.0 as usize),
                        LPARAM::default(),
                    )
                    .unwrap();
                } else {
                    PostMessageA(
                        None,
                        messages::STOP_MANAGING_WINDOW,
                        WPARAM(hwnd.0 as usize),
                        LPARAM::default(),
                    )
                    .unwrap();
                }
            }
            EVENT_OBJECT_HIDE if idobject == OBJID_WINDOW.0 => {
                PostMessageA(
                    None,
                    messages::STOP_MANAGING_WINDOW,
                    WPARAM(hwnd.0 as usize),
                    LPARAM::default(),
                )
                .unwrap();
            }
            EVENT_OBJECT_CLOAKED if idobject == OBJID_WINDOW.0 => {
                PostMessageA(
                    None,
                    messages::WINDOW_CLOAKED,
                    WPARAM(hwnd.0 as usize),
                    LPARAM::default(),
                )
                .unwrap();
            }
            EVENT_OBJECT_UNCLOAKED if idobject == OBJID_WINDOW.0 => {
                PostMessageA(
                    None,
                    messages::WINDOW_UNCLOAKED,
                    WPARAM(hwnd.0 as usize),
                    LPARAM::default(),
                )
                .unwrap();
            }
            EVENT_SYSTEM_FOREGROUND | EVENT_OBJECT_FOCUS => {
                PostMessageA(
                    None,
                    messages::FOREGROUND_WINDOW_CHANGED,
                    WPARAM(hwnd.0 as usize),
                    LPARAM::default(),
                )
                .unwrap();
            }
            EVENT_SYSTEM_MOVESIZEEND => {
                PostMessageA(
                    None,
                    messages::WINDOW_MOVE_FINISHED,
                    WPARAM(hwnd.0 as usize),
                    LPARAM::default(),
                )
                .unwrap();
            }
            _ => return,
        }
    }

    unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let wm = &mut *(lparam.0 as *mut WindowManager);
        let desktop_id = match wm.virtual_desktop_manager.GetWindowDesktopId(hwnd) {
            Ok(guid) if guid != GUID::zeroed() => guid,
            _ => return true.into(),
        };
        let monitor_handle = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONULL);
        if monitor_handle.is_invalid() {
            return true.into();
        }
        if !IsWindowVisible(hwnd).as_bool() || !is_overlappedwindow(hwnd) {
            return true.into();
        }
        wm.window_info.insert(
            hwnd.0,
            WindowInfo::new(desktop_id, monitor_handle, is_restored(hwnd), 0),
        );
        wm.push_hwnd(desktop_id, monitor_handle, hwnd);
        wm.initialize_border(hwnd);
        return true.into();
    }

    unsafe extern "system" fn enum_display_monitors_callback(
        hmonitor: HMONITOR,
        _hdc: HDC,
        _hdc_monitor: *mut RECT,
        dw_data: LPARAM,
    ) -> BOOL {
        let wm = &mut *(dw_data.0 as *mut WindowManager);
        wm.monitor_handles.push(hmonitor);
        wm.layouts.insert(hmonitor.0, Vec::new());
        return true.into();
    }
}

unsafe fn is_restored(hwnd: HWND) -> bool {
    return has_sizebox(hwnd)
        && !IsIconic(hwnd).as_bool()
        && !IsZoomed(hwnd).as_bool()
        && !IsWindowArranged(hwnd).as_bool()
        && IsWindowVisible(hwnd).as_bool();
}

unsafe fn has_sizebox(hwnd: HWND) -> bool {
    GetWindowLongPtrA(hwnd, GWL_STYLE) & WS_SIZEBOX.0 as isize != 0
}

unsafe fn is_overlappedwindow(hwnd: HWND) -> bool {
    GetWindowLongPtrA(hwnd, GWL_STYLE) & WS_OVERLAPPEDWINDOW.0 as isize != 0
}

pub unsafe fn convert_for_monitor(layout: &Layout, hmonitor: HMONITOR) -> Option<Layout> {
    let mut monitor_info = MONITORINFO::default();
    monitor_info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
    let _ = GetMonitorInfoA(hmonitor, &mut monitor_info);
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

pub unsafe fn register_hotkeys() {
    let _focus_previous = RegisterHotKey(
        None,
        hotkey_identifiers::FOCUS_PREVIOUS as i32,
        MOD_ALT,
        0x4A,
    );
    let _focus_next = RegisterHotKey(None, hotkey_identifiers::FOCUS_NEXT as i32, MOD_ALT, 0x4B);
    let _swap_previous = RegisterHotKey(
        None,
        hotkey_identifiers::SWAP_PREVIOUS as i32,
        MOD_ALT,
        0x48,
    );
    let _swap_next = RegisterHotKey(None, hotkey_identifiers::SWAP_NEXT as i32, MOD_ALT, 0x4C);
    let _variant_previous = RegisterHotKey(
        None,
        hotkey_identifiers::VARIANT_PREVIOUS as i32,
        MOD_ALT | MOD_SHIFT,
        0x4A,
    );
    let _variant_next = RegisterHotKey(
        None,
        hotkey_identifiers::VARIANT_NEXT as i32,
        MOD_ALT | MOD_SHIFT,
        0x4B,
    );
    let _layout_previous = RegisterHotKey(
        None,
        hotkey_identifiers::LAYOUT_PREVIOUS as i32,
        MOD_ALT | MOD_SHIFT,
        0x48,
    );
    let _layout_next = RegisterHotKey(
        None,
        hotkey_identifiers::LAYOUT_NEXT as i32,
        MOD_ALT | MOD_SHIFT,
        0x4C,
    );
    let _focus_previous_monitor = RegisterHotKey(
        None,
        hotkey_identifiers::FOCUS_PREVIOUS_MONITOR as i32,
        MOD_ALT,
        0x55,
    );
    let _focus_next_monitor = RegisterHotKey(
        None,
        hotkey_identifiers::FOCUS_NEXT_MONITOR as i32,
        MOD_ALT,
        0x49,
    );
    let _move_to_previous_monitor = RegisterHotKey(
        None,
        hotkey_identifiers::MOVE_TO_PREVIOUS_MONITOR as i32,
        MOD_ALT,
        0x59,
    );
    let _move_to_next_monitor = RegisterHotKey(
        None,
        hotkey_identifiers::MOVE_TO_NEXT_MONITOR as i32,
        MOD_ALT,
        0x4F,
    );
    let _grab_window = RegisterHotKey(
        None,
        hotkey_identifiers::GRAB_WINDOW as i32,
        MOD_ALT | MOD_SHIFT | MOD_NOREPEAT,
        0x55,
    );
    let _release_window = RegisterHotKey(
        None,
        hotkey_identifiers::RELEASE_WINDOW as i32,
        MOD_ALT | MOD_SHIFT | MOD_NOREPEAT,
        0x49,
    );
    let _toggle_window = RegisterHotKey(
        None,
        hotkey_identifiers::TOGGLE_WINDOW as i32,
        MOD_ALT | MOD_SHIFT | MOD_NOREPEAT,
        0x59,
    );
    let _toggle_workspace = RegisterHotKey(
        None,
        hotkey_identifiers::TOGGLE_WORKSPACE as i32,
        MOD_ALT | MOD_SHIFT | MOD_NOREPEAT,
        0x4F,
    );
}

pub unsafe fn handle_message(msg: MSG, wm: &mut WindowManager) {
    match msg.message {
        messages::WINDOW_CREATED => {
            wm.manage_window(HWND(msg.wParam.0 as *mut core::ffi::c_void));
        }
        messages::WINDOW_RESTORED
            if wm
                .window_info
                .contains_key(&(msg.wParam.0 as *mut core::ffi::c_void)) =>
        {
            wm.manage_window(HWND(msg.wParam.0 as *mut core::ffi::c_void));
        }
        messages::WINDOW_DESTROYED => {
            wm.window_destroyed(HWND(msg.wParam.0 as *mut core::ffi::c_void));
        }
        messages::STOP_MANAGING_WINDOW => {
            wm.stop_managing_window(HWND(msg.wParam.0 as *mut core::ffi::c_void));
        }
        messages::WINDOW_CLOAKED => {
            wm.window_cloaked(HWND(msg.wParam.0 as *mut core::ffi::c_void));
        }
        messages::WINDOW_UNCLOAKED => {
            wm.window_uncloaked(HWND(msg.wParam.0 as *mut core::ffi::c_void));
        }
        messages::FOREGROUND_WINDOW_CHANGED => {
            wm.foreground_window_changed(HWND(msg.wParam.0 as *mut core::ffi::c_void), false);
        }
        messages::WINDOW_MOVE_FINISHED => {
            wm.window_move_finished(HWND(msg.wParam.0 as *mut core::ffi::c_void));
        }
        WM_HOTKEY => match msg.wParam.0 {
            hotkey_identifiers::FOCUS_PREVIOUS => {
                wm.cycle_focus(CycleDirection::Previous);
            }
            hotkey_identifiers::FOCUS_NEXT => {
                wm.cycle_focus(CycleDirection::Next);
            }
            hotkey_identifiers::SWAP_PREVIOUS => {
                wm.cycle_swap(CycleDirection::Previous);
            }
            hotkey_identifiers::SWAP_NEXT => {
                wm.cycle_swap(CycleDirection::Next);
            }
            hotkey_identifiers::VARIANT_PREVIOUS => {
                wm.cycle_variant(CycleDirection::Previous);
            }
            hotkey_identifiers::VARIANT_NEXT => {
                wm.cycle_variant(CycleDirection::Next);
            }
            hotkey_identifiers::LAYOUT_PREVIOUS => {
                wm.cycle_layout(CycleDirection::Previous);
            }
            hotkey_identifiers::LAYOUT_NEXT => {
                wm.cycle_layout(CycleDirection::Next);
            }
            hotkey_identifiers::FOCUS_PREVIOUS_MONITOR => {
                wm.cycle_focused_monitor(CycleDirection::Previous);
            }
            hotkey_identifiers::FOCUS_NEXT_MONITOR => {
                wm.cycle_focused_monitor(CycleDirection::Next);
            }
            hotkey_identifiers::MOVE_TO_PREVIOUS_MONITOR => {
                wm.cycle_assigned_monitor(CycleDirection::Previous);
            }
            hotkey_identifiers::MOVE_TO_NEXT_MONITOR => {
                wm.cycle_assigned_monitor(CycleDirection::Next);
            }
            hotkey_identifiers::GRAB_WINDOW => {
                wm.grab_window();
            }
            hotkey_identifiers::RELEASE_WINDOW => {
                wm.release_window();
            }
            hotkey_identifiers::TOGGLE_WINDOW => {
                wm.toggle_window();
            }
            hotkey_identifiers::TOGGLE_WORKSPACE => {
                wm.toggle_workspace();
            }
            _ => (),
        },
        _ => (),
    }
}

pub unsafe fn show_error_message(message: &str) {
    let _free_console = FreeConsole();
    let _alloc_console = AllocConsole();
    let handle = GetStdHandle(STD_INPUT_HANDLE).unwrap();
    let mut console_mode = CONSOLE_MODE::default();
    let _get_console_mode = GetConsoleMode(handle, &mut console_mode);
    let _set_console_mode = SetConsoleMode(handle, console_mode & !ENABLE_ECHO_INPUT);
    println!("{}", message);
    println!("Press ENTER to exit");
    let mut buf = String::new();
    let _read_line = std::io::stdin().read_line(&mut buf);
}
