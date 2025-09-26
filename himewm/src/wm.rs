use crate::{window_rules, windows_api, wm_cb, wm_messages, wm_util};
use himewm_layout::layout::*;
use windows::{
    core::*,
    Win32::{
        Foundation::*,
        Graphics::{Dwm::*, Gdi::*},
        System::Com::*,
        UI::{Accessibility::*, Shell::*, WindowsAndMessaging::*},
    },
};

pub enum CycleDirection {
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
    pub unfocused_border_colour: COLORREF,
    pub floating_window_default_w_ratio: f64,
    pub floating_window_default_h_ratio: f64,
    pub new_window_retries: i32,
}

impl Settings {
    fn get_unfocused_border_colour(&self) -> COLORREF {
        if self.disable_unfocused_border {
            return COLORREF(DWMWA_COLOR_NONE);
        } else {
            return self.unfocused_border_colour;
        }
    }
}

#[derive(Clone)]
struct Workspace {
    layout_idx: usize,
    variant_idx: Vec<usize>,
    window_handles: std::collections::HashSet<*mut core::ffi::c_void>,
    managed_window_handles: Vec<HWND>,
}

impl Workspace {
    fn new(hwnd: HWND, layout_idx: usize, variant_idx: Vec<usize>) -> Self {
        Self {
            layout_idx,
            variant_idx,
            window_handles: std::collections::HashSet::from([hwnd.0]),
            managed_window_handles: Vec::new(),
        }
    }

    fn insert_into_new(hwnd: HWND, layout_idx: usize, variant_idx: Vec<usize>) -> Self {
        Self {
            layout_idx,
            variant_idx,
            window_handles: std::collections::HashSet::from([hwnd.0]),
            managed_window_handles: vec![hwnd],
        }
    }
}

#[derive(Clone)]
pub struct WindowInfo {
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
    previous_foreground_window: Option<HWND>,
    grabbed_window: Option<HWND>,
    ignored_combinations: std::collections::HashSet<(GUID, *mut core::ffi::c_void)>,
    ignored_windows: std::collections::HashSet<*mut core::ffi::c_void>,
    desktop_switching_state: DesktopSwitchingState,
    settings: Settings,
    window_rules: window_rules::InternalWindowRules,
    restart_requested: bool,
}

impl WindowManager {
    pub fn new(
        settings: Settings,
        window_rules: window_rules::InternalWindowRules,
        existing_event_hook: Option<HWINEVENTHOOK>,
        existing_vd_manager: Option<IVirtualDesktopManager>,
    ) -> Self {
        let event_hook = match existing_event_hook {
            Some(hook) => hook,
            None => windows_api::set_win_event_hook(
                EVENT_MIN,
                EVENT_MAX,
                None,
                Some(wm_cb::event_handler),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            ),
        };
        let virtual_desktop_manager = match existing_vd_manager {
            Some(vd_manager) => vd_manager,
            None => {
                let _ = windows_api::co_initialize_ex(None, COINIT_MULTITHREADED);
                windows_api::co_create_instance(&VirtualDesktopManager, None, CLSCTX_INPROC_SERVER)
                    .unwrap()
            }
        };
        Self {
            event_hook,
            virtual_desktop_manager,
            monitor_handles: Vec::new(),
            window_info: std::collections::HashMap::new(),
            workspaces: std::collections::HashMap::new(),
            layouts: std::collections::HashMap::new(),
            foreground_window: None,
            previous_foreground_window: None,
            grabbed_window: None,
            ignored_combinations: std::collections::HashSet::new(),
            ignored_windows: std::collections::HashSet::new(),
            desktop_switching_state: DesktopSwitchingState::default(),
            settings,
            window_rules,
            restart_requested: false,
        }
    }

    pub fn initialize(&mut self, layouts: Vec<Layout>) {
        let _ = windows_api::enum_display_monitors(
            None,
            None,
            Some(wm_cb::enum_display_monitors_callback),
            LPARAM(self as *mut WindowManager as isize),
        );
        for layout in layouts {
            for (hmonitor, wm_layouts) in self.layouts.iter_mut() {
                let mut layout =
                    match wm_util::convert_layout_for_monitor(&layout, HMONITOR(*hmonitor)) {
                        Some(val) => val,
                        None => layout.clone(),
                    };
                layout.update_all(self.settings.window_padding, self.settings.edge_padding);
                wm_layouts.push(layout);
            }
        }
        let _ = windows_api::enum_windows(
            Some(wm_cb::enum_windows_callback),
            LPARAM(self as *mut WindowManager as isize),
        );
        let foreground_window = windows_api::get_foreground_window();
        if self.window_info.contains_key(&foreground_window.0) {
            self.foreground_window = Some(foreground_window);
            self.set_border_to_focused(foreground_window);
        }
        self.update();
    }

    pub fn get_event_hook(self) -> HWINEVENTHOOK {
        self.event_hook
    }

    pub fn get_virtual_desktop_manager(&self) -> &IVirtualDesktopManager {
        &self.virtual_desktop_manager
    }

    pub fn get_monitor_vec(&self) -> &Vec<HMONITOR> {
        &self.monitor_handles
    }

    pub fn get_monitor_vec_mut(&mut self) -> &mut Vec<HMONITOR> {
        &mut self.monitor_handles
    }

    pub fn get_window_info_hashmap(
        &self,
    ) -> &std::collections::HashMap<*mut core::ffi::c_void, WindowInfo> {
        &self.window_info
    }

    pub fn get_window_info_hashmap_mut(
        &mut self,
    ) -> &mut std::collections::HashMap<*mut core::ffi::c_void, WindowInfo> {
        &mut self.window_info
    }

    pub fn get_layouts_hashmap(
        &self,
    ) -> &std::collections::HashMap<*mut core::ffi::c_void, Vec<Layout>> {
        &self.layouts
    }

    pub fn get_layouts_hashmap_mut(
        &mut self,
    ) -> &mut std::collections::HashMap<*mut core::ffi::c_void, Vec<Layout>> {
        &mut self.layouts
    }

    pub fn get_settings(&self) -> &Settings {
        &self.settings
    }

    pub fn get_settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    pub fn restart_requested(&self) -> bool {
        self.restart_requested
    }

    pub fn manage_new_window(&mut self, guid: GUID, hmonitor: HMONITOR, hwnd: HWND) {
        self.window_info.insert(
            hwnd.0,
            WindowInfo::new(guid, hmonitor, wm_util::is_restored(hwnd), 0),
        );
        let filter = Some(std::collections::HashSet::from([
            window_rules::FilterRule::Layout,
            window_rules::FilterRule::StartFloating,
        ]));
        match self.get_window_rule(hwnd, &filter) {
            Some(rule) => match rule {
                window_rules::Rule::LayoutIdx(idx) => {
                    self.push_hwnd(guid, hmonitor, hwnd);
                    if let Some(workspace) = self.workspaces.get_mut(&(guid, hmonitor.0)) {
                        workspace.layout_idx = idx;
                    }
                }
                window_rules::Rule::StartFloating(set_position) => {
                    self.add_hwnd_to_workspace(guid, hmonitor, hwnd);
                    self.ignored_windows.insert(hwnd.0);
                    match set_position {
                        window_rules::SetPosition::Default => (),
                        window_rules::SetPosition::Center => self.center_window(hwnd),
                        window_rules::SetPosition::Position(window_rules::Position {
                            x,
                            y,
                            w,
                            h,
                        }) => {
                            let _ =
                                windows_api::set_window_pos(hwnd, None, x, y, w, h, SWP_NOZORDER);
                        }
                    }
                }
                _ => (),
            },
            None => self.push_hwnd(guid, hmonitor, hwnd),
        }
        self.initialize_border(hwnd);
    }

    pub fn manage_window(&mut self, hwnd: HWND) {
        let desktop_id;
        let monitor_handle;
        match self.window_info.get_mut(&hwnd.0) {
            Some(window_info) if window_info.restored => return,
            Some(window_info) if wm_util::is_restored(hwnd) => {
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
                    match windows_api::get_window_desktop_id(&self.virtual_desktop_manager, hwnd) {
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
                monitor_handle = windows_api::monitor_from_window(hwnd, MONITOR_DEFAULTTONULL);
                if monitor_handle.is_invalid() {
                    return;
                }
                self.manage_new_window(desktop_id, monitor_handle, hwnd);
                if let None = self.foreground_window {
                    self.foreground_window_changed(hwnd, false);
                }
            }
            _ => return,
        }
        self.update_workspace(desktop_id, monitor_handle);
    }

    pub fn window_destroyed(&mut self, hwnd: HWND) {
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
            ..
        } = window_info.to_owned();
        self.remove_hwnd(hwnd);
        if let Some(workspace) = self.workspaces.get(&(desktop_id, monitor_handle.0)) {
            if workspace.window_handles.len() == 0 {
                self.workspaces.remove(&(desktop_id, monitor_handle.0));
            } else if restored && !self.ignored_windows.contains(&hwnd.0) {
                self.update_workspace(desktop_id, monitor_handle);
            }
        }
    }

    pub fn stop_managing_window(&mut self, hwnd: HWND) {
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
        if self.foreground_window == Some(hwnd) {
            self.foreground_window = None;
        }
        if self.previous_foreground_window == Some(hwnd) {
            self.previous_foreground_window = None;
        }
        if self.grabbed_window == Some(hwnd) && !self.ignored_windows.contains(&hwnd.0) {
            self.grabbed_window = None;
        }
        if !self.ignored_windows.contains(&hwnd.0) {
            self.unmanage_hwnd(desktop_id, monitor_handle, idx, false);
            self.update_workspace(desktop_id, monitor_handle);
        }
    }

    pub fn window_cloaked(&mut self, hwnd: HWND) {
        let window_info = match self.window_info.get(&hwnd.0) {
            Some(val) => val,
            None => return,
        };
        self.desktop_switching_state.uncloak_count = 0;
        self.desktop_switching_state.max_uncloak_count = 0;
        let WindowInfo {
            desktop_id: old_desktop_id,
            monitor_handle,
            restored,
            ..
        } = window_info.to_owned();
        let new_desktop_id =
            match windows_api::get_window_desktop_id(&self.virtual_desktop_manager, hwnd) {
                Ok(guid) if guid != old_desktop_id => guid,
                _ => return,
            };
        self.remove_hwnd_from_workspace(hwnd);
        if restored && !self.ignored_windows.contains(&hwnd.0) {
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
        if let Some(old_workspace) = self.workspaces.get(&(old_desktop_id, monitor_handle.0)) {
            if old_workspace.window_handles.len() == 0 {
                self.workspaces.remove(&(old_desktop_id, monitor_handle.0));
            } else {
                self.update_workspace(old_desktop_id, monitor_handle);
            }
        }
        self.update_workspace(new_desktop_id, monitor_handle);
    }

    pub fn window_uncloaked(&mut self, hwnd: HWND) {
        let window_info = match self.window_info.get(&hwnd.0) {
            Some(val) => val,
            None => return,
        };
        let WindowInfo {
            desktop_id: uncloaked_desktop_id,
            monitor_handle: current_monitor_handle,
            restored,
            ..
        } = window_info.to_owned();
        if self.desktop_switching_state.uncloak_count
            == self.desktop_switching_state.max_uncloak_count
        {
            self.desktop_switching_state.uncloak_count = 0;
            self.desktop_switching_state.max_uncloak_count = 0;
        }
        if !restored && self.desktop_switching_state.max_uncloak_count != 0 {
            return;
        }
        if self.desktop_switching_state.uncloak_count == 0 {
            let foreground_hwnd = match self.foreground_window {
                Some(h) if h != hwnd => h,
                Some(_) => return,
                None => {
                    match self
                        .workspaces
                        .get(&(uncloaked_desktop_id, current_monitor_handle.0))
                    {
                        Some(w) if w.managed_window_handles.len() > 0 => {
                            let _ = windows_api::set_foreground_window(w.managed_window_handles[0]);
                        }
                        _ => (),
                    }
                    return;
                }
            };
            let previous_desktop_id = self.window_info.get(&foreground_hwnd.0).unwrap().desktop_id;
            let mut gathered_hwnds_and_indices = Vec::new();
            for monitor_handle in self.monitor_handles.to_owned() {
                if let Some(workspace) = self
                    .workspaces
                    .get(&(uncloaked_desktop_id, monitor_handle.0))
                {
                    self.desktop_switching_state.max_uncloak_count +=
                        workspace.managed_window_handles.len();
                }
                if let Some(workspace) = self
                    .workspaces
                    .get(&(previous_desktop_id, monitor_handle.0))
                {
                    for h in &workspace.window_handles {
                        let idx = self.window_info.get(h).unwrap().idx;
                        gathered_hwnds_and_indices.push((*h, idx));
                    }
                }
            }
            if restored {
                self.desktop_switching_state.uncloak_count += 1;
            }
            let mut new_desktop_id = None;
            gathered_hwnds_and_indices.sort_by(|a, b| a.1.cmp(&b.1));
            let gathered_hwnds = gathered_hwnds_and_indices
                .iter()
                .map(|(h, _idx)| *h)
                .collect::<Vec<*mut core::ffi::c_void>>();
            for h in gathered_hwnds {
                let info = self.window_info.get(&h).unwrap().to_owned();
                match windows_api::get_window_desktop_id(&self.virtual_desktop_manager, HWND(h)) {
                    Ok(guid) if guid != previous_desktop_id => {
                        self.remove_hwnd_from_workspace(HWND(h));
                        if info.restored && !self.ignored_windows.contains(&h) {
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
                    if let Some(previous_workspace) = self
                        .workspaces
                        .get(&(previous_desktop_id, monitor_handle.0))
                    {
                        if previous_workspace.window_handles.len() == 0 {
                            self.workspaces
                                .remove(&(previous_desktop_id, monitor_handle.0));
                        } else {
                            self.update_workspace(previous_desktop_id, monitor_handle);
                        }
                    }
                    self.update_workspace(guid, monitor_handle);
                }
            }
        } else {
            self.desktop_switching_state.uncloak_count += 1;
        }
    }

    pub fn foreground_window_changed(&mut self, hwnd: HWND, updating: bool) {
        if !self.window_info.contains_key(&hwnd.0) {
            if let Some(previous_foreground_window) = self.foreground_window {
                self.previous_foreground_window = Some(previous_foreground_window);
                self.unfocus_border_with_combination_check(previous_foreground_window);
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
        let ignored_combination = self
            .ignored_combinations
            .contains(&(desktop_id, monitor_handle.0));
        if !ignored_combination {
            self.set_border_to_focused(hwnd);
        }
        match self.foreground_window {
            Some(prev) if prev == hwnd => {
                if let Some(previous_foreground_window) = self.previous_foreground_window {
                    self.unfocus_border_with_combination_check(previous_foreground_window);
                }
                if !updating {
                    return;
                }
            }
            Some(previous_foreground_window) => {
                self.previous_foreground_window = Some(previous_foreground_window);
                self.unfocus_border_with_combination_check(previous_foreground_window);
            }
            None => {
                self.previous_foreground_window = None;
            }
        }
        self.foreground_window = Some(hwnd);
        if ignored_combination {
            return;
        }
        if !self.ignored_windows.contains(&hwnd.0) && wm_util::is_restored(hwnd) {
            if let Some(workspace) = self.workspaces.get(&(desktop_id, monitor_handle.0)) {
                for h in &workspace.window_handles {
                    let info = self.window_info.get(h).unwrap();
                    if h != &hwnd.0 && (!info.restored || self.ignored_windows.contains(h)) {
                        let _ = windows_api::show_window(HWND(*h), SW_MINIMIZE);
                    }
                }
            }
        }
    }

    pub fn window_move_finished(&mut self, hwnd: HWND) {
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
        let new_monitor_handle = windows_api::monitor_from_window(hwnd, MONITOR_DEFAULTTONULL);
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
        windows_api::get_window_rect(hwnd, &mut moved_to).unwrap();
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
                        Workspace::insert_into_new(
                            hwnd,
                            self.settings.default_layout_idx,
                            self.layouts.get(&new_monitor_handle.0).unwrap()
                                [self.settings.default_layout_idx]
                                .default_variant_idx()
                                .to_owned(),
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
            let layout;
            let positions = if changed_monitors {
                layout =
                    &mut self.layouts.get_mut(&new_monitor_handle.0).unwrap()[workspace.layout_idx];
                layout.get_internal_positions(
                    &workspace.variant_idx,
                    workspace.managed_window_handles.len() + 1,
                    self.settings.window_padding,
                    self.settings.edge_padding,
                )
            } else {
                layout = &mut self.layouts.get_mut(&original_monitor_handle.0).unwrap()
                    [workspace.layout_idx];
                layout.get_internal_positions(
                    &workspace.variant_idx,
                    workspace.managed_window_handles.len(),
                    self.settings.window_padding,
                    self.settings.edge_padding,
                )
            };
            if !changed_monitors {
                let position = &positions[idx];
                if moved_to.left == position.x()
                    && moved_to.top == position.y()
                    && moved_to.right - moved_to.left == position.w()
                    && moved_to.bottom - moved_to.top == position.h()
                {
                    return;
                }
            }
            for (i, p) in positions.iter().enumerate() {
                let left = std::cmp::max(moved_to.left, p.x());
                let top = std::cmp::max(moved_to.top, p.y());
                let right = std::cmp::min(moved_to.right, p.x() + p.w());
                let bottom = std::cmp::min(moved_to.bottom, p.y() + p.h());
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

    pub fn cycle_focus(&self, direction: CycleDirection) {
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
        let _ = windows_api::set_foreground_window(workspace.managed_window_handles[to]);
    }

    pub fn cycle_swap(&mut self, direction: CycleDirection) {
        let foreground_window = match self.foreground_window {
            Some(hwnd) => hwnd,
            None => return,
        };
        if self.ignored_windows.contains(&foreground_window.0) {
            return;
        }
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

    pub fn cycle_variant(&mut self, direction: CycleDirection, idx: usize) {
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
        while workspace.variant_idx.len() <= idx {
            workspace.variant_idx.push(0);
        }
        let variants_len = match self.layouts.get(&monitor_handle.0).unwrap()[workspace.layout_idx]
            .get_variants()
            .get(&workspace.variant_idx[0..idx])
        {
            himewm_layout::variants_container::VariantsContainerReturn::Container(container) => {
                container.len()
            }
            himewm_layout::variants_container::VariantsContainerReturn::Variant(_) => return,
        };
        if variants_len == 1 {
            workspace.variant_idx[idx] = 0;
            return;
        }
        if workspace.variant_idx[idx] > variants_len - 1 {
            workspace.variant_idx[idx] = variants_len - 1;
        }
        match direction {
            CycleDirection::Previous => {
                if workspace.variant_idx[idx] != 0 {
                    workspace.variant_idx[idx] -= 1;
                } else {
                    workspace.variant_idx[idx] = variants_len - 1;
                }
            }
            CycleDirection::Next => {
                if workspace.variant_idx[idx] != variants_len - 1 {
                    workspace.variant_idx[idx] += 1;
                } else {
                    workspace.variant_idx[idx] = 0;
                }
            }
        }
        self.update_workspace(desktop_id, monitor_handle);
    }

    pub fn cycle_layout(&mut self, direction: CycleDirection) {
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
        workspace.variant_idx = layouts[workspace.layout_idx]
            .default_variant_idx()
            .to_owned();
        self.update_workspace(desktop_id, monitor_handle);
    }

    pub fn cycle_focused_monitor(&self, direction: CycleDirection) {
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
        let _ = windows_api::set_foreground_window(workspace.managed_window_handles[0]);
    }

    pub fn cycle_assigned_monitor(&mut self, direction: CycleDirection) {
        if self.monitor_handles.len() <= 1 {
            return;
        }
        let foreground_window = match self.foreground_window {
            Some(hwnd) if !self.ignored_windows.contains(&hwnd.0) => hwnd,
            _ => return,
        };
        let original_dpi = windows_api::get_dpi_for_window(foreground_window);
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
                self.remove_hwnd_from_workspace(foreground_window);
                // self.unmanage_hwnd(desktop_id, original_monitor_handle, original_window_idx);
                self.workspaces.insert(
                    (desktop_id, new_monitor_handle.0),
                    Workspace::insert_into_new(
                        foreground_window,
                        self.settings.default_layout_idx,
                        self.layouts.get(&new_monitor_handle.0).unwrap()
                            [self.settings.default_layout_idx]
                            .default_variant_idx()
                            .to_owned(),
                    ),
                );
                let window_info_mut = self.window_info.get_mut(&foreground_window.0).unwrap();
                window_info_mut.monitor_handle = new_monitor_handle;
                window_info_mut.idx = 0;
            }
        };
        self.update_workspace(desktop_id, original_monitor_handle);
        self.update_workspace(desktop_id, new_monitor_handle);
        if windows_api::get_dpi_for_window(foreground_window) != original_dpi {
            let workspace = self
                .workspaces
                .get(&(desktop_id, new_monitor_handle.0))
                .unwrap();
            let layout =
                &mut self.layouts.get_mut(&new_monitor_handle.0).unwrap()[workspace.layout_idx];
            let position = &layout.get_internal_positions(
                &workspace.variant_idx,
                workspace.managed_window_handles.len(),
                self.settings.window_padding,
                self.settings.edge_padding,
            )[workspace.managed_window_handles.len() - 1];
            let _ = windows_api::set_window_pos(
                foreground_window,
                None,
                position.x(),
                position.y(),
                position.w(),
                position.h(),
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

    pub fn release_window(&mut self) {
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
            let _ = windows_api::show_window(grabbed_window, SW_RESTORE);
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
            let original_dpi = windows_api::get_dpi_for_window(grabbed_window);
            self.update_workspace(original_desktop_id, original_monitor_handle);
            self.update_workspace(original_desktop_id, new_monitor_handle);
            if windows_api::get_dpi_for_window(grabbed_window) != original_dpi {
                let workspace = self
                    .workspaces
                    .get(&(original_desktop_id, new_monitor_handle.0))
                    .unwrap();
                let layout =
                    &mut self.layouts.get_mut(&new_monitor_handle.0).unwrap()[workspace.layout_idx];
                let position = &layout.get_internal_positions(
                    &workspace.variant_idx,
                    workspace.managed_window_handles.len(),
                    self.settings.window_padding,
                    self.settings.edge_padding,
                )[new_idx];
                let _ = windows_api::set_window_pos(
                    grabbed_window,
                    None,
                    position.x(),
                    position.y(),
                    position.w(),
                    position.h(),
                    SWP_NOZORDER,
                );
            }
        }
        let _ = windows_api::set_foreground_window(grabbed_window);
        self.grabbed_window = None;
    }

    pub fn toggle_window(&mut self) {
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
        if self
            .ignored_combinations
            .contains(&(desktop_id, monitor_handle.0))
        {
            return;
        }
        if self.ignored_windows.remove(&foreground_window.0) {
            if restored {
                let original_dpi = windows_api::get_dpi_for_window(foreground_window);
                self.foreground_window_changed(foreground_window, true);
                self.insert_hwnd(desktop_id, monitor_handle, idx, foreground_window);
                self.update_workspace(desktop_id, monitor_handle);
                if windows_api::get_dpi_for_window(foreground_window) != original_dpi {
                    let workspace = self
                        .workspaces
                        .get(&(desktop_id, monitor_handle.0))
                        .unwrap();
                    let layout =
                        &mut self.layouts.get_mut(&monitor_handle.0).unwrap()[workspace.layout_idx];
                    let position = &layout.get_internal_positions(
                        &workspace.variant_idx,
                        workspace.managed_window_handles.len(),
                        self.settings.window_padding,
                        self.settings.edge_padding,
                    )[workspace.managed_window_handles.len() - 1];
                    let _ = windows_api::set_window_pos(
                        foreground_window,
                        None,
                        position.x(),
                        position.y(),
                        position.w(),
                        position.h(),
                        SWP_NOZORDER,
                    );
                }
            }
        } else {
            self.ignored_windows.insert(foreground_window.0);
            if let None = self.unmanage_hwnd(desktop_id, monitor_handle, idx, false) {
                return;
            }
            self.update_workspace(desktop_id, monitor_handle);
            let filter = Some(std::collections::HashSet::from([
                window_rules::FilterRule::FloatingPosition,
            ]));
            match self.get_window_rule(foreground_window, &filter) {
                Some(rule) => match rule {
                    window_rules::Rule::FloatingPosition(window_rules::Position { x, y, w, h }) => {
                        let _ = windows_api::set_window_pos(
                            foreground_window,
                            None,
                            x,
                            y,
                            w,
                            h,
                            SWP_NOZORDER,
                        );
                    }
                    _ => (),
                },
                None => self.center_window(foreground_window),
            }
        }
    }

    pub fn toggle_workspace(&mut self) {
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
        let workspace = match self.workspaces.get(&(*desktop_id, monitor_handle.0)) {
            Some(w) => w,
            None => return,
        };
        if self
            .ignored_combinations
            .remove(&(*desktop_id, monitor_handle.0))
        {
            for h in &workspace.window_handles {
                self.initialize_border(HWND(*h));
            }
            self.set_border_to_focused(foreground_window);
            self.update_workspace(*desktop_id, *monitor_handle);
        } else {
            for h in &workspace.window_handles {
                Self::reset_border(HWND(*h));
            }
            self.ignored_combinations
                .insert((*desktop_id, monitor_handle.0));
        }
    }

    pub fn refresh_workspace(&mut self) {
        let foreground_window = match self.foreground_window {
            Some(hwnd) => hwnd,
            None => return,
        };
        let WindowInfo {
            desktop_id,
            monitor_handle,
            ..
        } = match self.window_info.get(&foreground_window.0) {
            Some(val) => val.to_owned(),
            None => return,
        };
        if self
            .ignored_combinations
            .contains(&(desktop_id, monitor_handle.0))
        {
            return;
        }
        self.update_workspace(desktop_id, monitor_handle);
    }

    pub fn restart_himewm(&mut self) {
        self.restart_requested = true;
        windows_api::post_message(
            None,
            wm_messages::messages::RESTART_HIMEWM,
            WPARAM::default(),
            LPARAM::default(),
        )
        .unwrap();
    }

    fn update_workspace(&mut self, guid: GUID, hmonitor: HMONITOR) {
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
        let mut error_indices: Option<Vec<usize>> = None;
        let positions = layout.get_internal_positions(
            &workspace.variant_idx,
            workspace.managed_window_handles.len(),
            self.settings.window_padding,
            self.settings.edge_padding,
        );
        for (i, hwnd) in workspace.managed_window_handles.iter().enumerate() {
            match windows_api::set_window_pos(
                *hwnd,
                None,
                positions[i].x(),
                positions[i].y(),
                positions[i].w(),
                positions[i].h(),
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
                    if windows_api::get_last_error().0 == 5 {
                        self.ignored_windows.insert(hwnd.0);
                    }
                }
            }
        }
        if let Some(v) = error_indices {
            for (i, error_idx) in v.iter().enumerate() {
                self.unmanage_hwnd(guid, hmonitor, *error_idx - i, true);
            }
            self.update_workspace(guid, hmonitor);
        }
    }

    fn update(&mut self) {
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

    fn move_windows_across_monitors(
        &mut self,
        guid: GUID,
        first_hmonitor: HMONITOR,
        second_hmonitor: HMONITOR,
        first_idx: usize,
        second_idx: usize,
    ) {
        let hwnd = match self.unmanage_hwnd(guid, first_hmonitor, first_idx, true) {
            Some(val) => val,
            None => return,
        };
        self.insert_hwnd(guid, second_hmonitor, second_idx, hwnd);
    }

    fn set_border_to_unfocused(&self, hwnd: HWND) {
        let _ = windows_api::dwm_set_window_attribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &self.settings.get_unfocused_border_colour() as *const COLORREF
                as *const core::ffi::c_void,
            std::mem::size_of_val(&self.settings.get_unfocused_border_colour()) as u32,
        );
    }

    fn set_border_to_focused(&self, hwnd: HWND) {
        let _ = windows_api::dwm_set_window_attribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &self.settings.focused_border_colour as *const COLORREF as *const core::ffi::c_void,
            std::mem::size_of_val(&self.settings.focused_border_colour) as u32,
        );
    }

    fn initialize_border(&self, hwnd: HWND) {
        let corner_preference = if self.settings.disable_rounding {
            DWMWCP_DONOTROUND
        } else {
            DWMWCP_DEFAULT
        };
        let _ = windows_api::dwm_set_window_attribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &corner_preference as *const DWM_WINDOW_CORNER_PREFERENCE as *const core::ffi::c_void,
            std::mem::size_of_val(&corner_preference) as u32,
        );
        self.set_border_to_unfocused(hwnd);
    }

    fn reset_border(hwnd: HWND) {
        let _corner = windows_api::dwm_set_window_attribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &DWMWCP_DEFAULT as *const DWM_WINDOW_CORNER_PREFERENCE as *const core::ffi::c_void,
            std::mem::size_of_val(&DWMWCP_DEFAULT) as u32,
        );
        let _border_colour = windows_api::dwm_set_window_attribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &COLORREF(DWMWA_COLOR_DEFAULT) as *const COLORREF as *const core::ffi::c_void,
            std::mem::size_of_val(&COLORREF(DWMWA_COLOR_DEFAULT)) as u32,
        );
    }

    fn unfocus_border_with_combination_check(&self, hwnd: HWND) {
        let WindowInfo {
            desktop_id,
            monitor_handle,
            ..
        } = match self.window_info.get(&hwnd.0) {
            Some(val) => val,
            None => return,
        };
        if !self
            .ignored_combinations
            .contains(&(*desktop_id, monitor_handle.0))
        {
            self.set_border_to_unfocused(hwnd);
        }
    }

    fn center_window(&self, hwnd: HWND) {
        let window_info = match self.window_info.get(&hwnd.0) {
            Some(val) => val,
            None => return,
        };
        let WindowInfo {
            desktop_id,
            monitor_handle,
            ..
        } = window_info.to_owned();
        let workspace = self
            .workspaces
            .get(&(desktop_id, monitor_handle.0))
            .unwrap();
        let monitor_rect =
            self.layouts.get(&monitor_handle.0).unwrap()[workspace.layout_idx].get_monitor_rect();
        let w = ((monitor_rect.w() as f64) * self.settings.floating_window_default_w_ratio).round()
            as i32;
        let h = ((monitor_rect.h() as f64) * self.settings.floating_window_default_h_ratio).round()
            as i32;
        let x = (((monitor_rect.w() - w) as f64) * 0.5).round() as i32 + monitor_rect.x();
        let y = (((monitor_rect.h() - h) as f64) * 0.5).round() as i32 + monitor_rect.y();
        let _ = windows_api::set_window_pos(hwnd, None, x - 7, y, w + 14, h + 7, SWP_NOZORDER);
    }

    fn add_hwnd_to_workspace(&mut self, guid: GUID, hmonitor: HMONITOR, hwnd: HWND) {
        let window_info = match self.window_info.get_mut(&hwnd.0) {
            Some(val) => val,
            None => return,
        };
        window_info.desktop_id = guid;
        window_info.monitor_handle = hmonitor;
        match self.workspaces.get_mut(&(guid, hmonitor.0)) {
            Some(workspace) => {
                workspace.window_handles.insert(hwnd.0);
                window_info.idx = workspace.managed_window_handles.len();
            }
            None => {
                self.workspaces.insert(
                    (guid, hmonitor.0),
                    Workspace::new(
                        hwnd,
                        self.settings.default_layout_idx,
                        self.layouts.get(&window_info.monitor_handle.0).unwrap()
                            [self.settings.default_layout_idx]
                            .default_variant_idx()
                            .to_owned(),
                    ),
                );
                window_info.idx = 0;
            }
        }
    }

    fn remove_hwnd_from_workspace(&mut self, hwnd: HWND) {
        let window_info = match self.window_info.get_mut(&hwnd.0) {
            Some(val) => val,
            None => return,
        };
        let WindowInfo {
            desktop_id,
            monitor_handle,
            restored,
            idx,
        } = window_info.to_owned();
        if let Some(workspace) = self.workspaces.get_mut(&(desktop_id, monitor_handle.0)) {
            workspace.window_handles.remove(&hwnd.0);
            if restored {
                self.unmanage_hwnd(desktop_id, monitor_handle, idx, false);
            }
        }
    }

    fn insert_hwnd(&mut self, guid: GUID, hmonitor: HMONITOR, idx: usize, hwnd: HWND) {
        let window_info = match self.window_info.get_mut(&hwnd.0) {
            Some(val) => val,
            None => return,
        };
        window_info.desktop_id = guid;
        window_info.monitor_handle = hmonitor;
        match self.workspaces.get_mut(&(guid, hmonitor.0)) {
            Some(workspace) => {
                workspace.window_handles.insert(hwnd.0);
                window_info.idx = idx;
                if window_info.restored {
                    workspace.managed_window_handles.insert(idx, hwnd);
                    for h in workspace.window_handles.to_owned() {
                        if windows_api::is_window(Some(HWND(h))).as_bool() {
                            let info = self.window_info.get_mut(&h).unwrap();
                            if h != hwnd.0 && info.idx >= idx {
                                info.idx += 1;
                            }
                        } else {
                            self.remove_hwnd(HWND(h));
                        }
                    }
                }
            }
            None => {
                if window_info.restored {
                    self.workspaces.insert(
                        (guid, hmonitor.0),
                        Workspace::insert_into_new(
                            hwnd,
                            self.settings.default_layout_idx,
                            self.layouts.get(&window_info.monitor_handle.0).unwrap()
                                [self.settings.default_layout_idx]
                                .default_variant_idx()
                                .to_owned(),
                        ),
                    );
                }
                window_info.idx = 0;
            }
        };
    }

    fn push_hwnd(&mut self, guid: GUID, hmonitor: HMONITOR, hwnd: HWND) {
        let idx = if let Some(workspace) = self.workspaces.get(&(guid, hmonitor.0)) {
            workspace.managed_window_handles.len()
        } else {
            0
        };
        self.insert_hwnd(guid, hmonitor, idx, hwnd);
    }

    fn unmanage_hwnd(
        &mut self,
        guid: GUID,
        hmonitor: HMONITOR,
        idx: usize,
        remove_from_workspace: bool,
    ) -> Option<HWND> {
        let workspace = match self.workspaces.get_mut(&(guid, hmonitor.0)) {
            Some(val) => val,
            None => return None,
        };
        let hwnd = workspace.managed_window_handles.remove(idx);
        if remove_from_workspace {
            workspace.window_handles.remove(&hwnd.0);
        }
        for h in workspace.window_handles.to_owned() {
            if windows_api::is_window(Some(HWND(h))).as_bool() {
                let info = self.window_info.get_mut(&h).unwrap();
                if info.idx > idx {
                    info.idx -= 1;
                }
            } else {
                self.remove_hwnd(HWND(h));
            }
        }
        return Some(hwnd);
    }

    fn remove_hwnd(&mut self, hwnd: HWND) {
        self.remove_hwnd_from_workspace(hwnd);
        self.window_info.remove(&hwnd.0);
        if self.foreground_window == Some(hwnd) {
            self.foreground_window = None;
        }
        if self.previous_foreground_window == Some(hwnd) {
            self.previous_foreground_window = None;
        }
        if self.grabbed_window == Some(hwnd) {
            self.grabbed_window = None;
        }
    }

    fn get_window_rule(
        &mut self,
        hwnd: HWND,
        filter: &Option<std::collections::HashSet<window_rules::FilterRule>>,
    ) -> Option<window_rules::Rule> {
        match wm_util::get_window_title(hwnd) {
            Ok(title) => {
                for window_rule in &self.window_rules.title_window_rules {
                    if window_rule.regex.is_match(&title) {
                        match &filter {
                            Some(f) => {
                                let filter_for = window_rules::FilterRule::from(&window_rule.rule);
                                if f.contains(&filter_for) {
                                    return Some(window_rule.rule.to_owned());
                                }
                            }
                            None => return Some(window_rule.rule.to_owned()),
                        }
                    }
                }
            }
            Err(_) => (),
        }
        match wm_util::get_exe_name(hwnd) {
            Ok(title) => {
                for window_rule in &self.window_rules.process_window_rules {
                    if window_rule.regex.is_match(&title) {
                        match &filter {
                            Some(f) => {
                                let filter_for = window_rules::FilterRule::from(&window_rule.rule);
                                if f.contains(&filter_for) {
                                    return Some(window_rule.rule.to_owned());
                                }
                            }
                            None => return Some(window_rule.rule.to_owned()),
                        }
                    }
                }
                return None;
            }
            Err(_) => return None,
        }
    }

    pub fn exit(self) {
        for h in self.window_info.keys() {
            Self::reset_border(HWND(*h));
        }
        let _unhook_win_event = windows_api::unhook_win_event(self.event_hook);
        windows_api::co_uninitialize();
    }
}
