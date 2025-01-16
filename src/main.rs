use windows::Win32::{

    Foundation::*, 
    
    UI::{

        Input::KeyboardAndMouse::*, 

        WindowsAndMessaging::*

    }

};

mod layout;
mod wm;

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

    pub const SWAP_PREVIOUS_MONITOR: usize = 10;

    pub const SWAP_NEXT_MONITOR: usize = 11;

    pub const GRAB_WINDOW: usize = 12;

    pub const RELEASE_WINDOW: usize = 13;

    pub const REFRESH_WORKSPACE: usize = 14;

    pub const TOGGLE_WORKSPACE: usize = 15;

}

unsafe fn register_hotkeys() {
    
    let _focus_previous = RegisterHotKey(None, hotkey_identifiers::FOCUS_PREVIOUS as i32, MOD_ALT, 0x4A);

    let _focus_next = RegisterHotKey(None, hotkey_identifiers::FOCUS_NEXT as i32, MOD_ALT, 0x4B);

    let _swap_previous = RegisterHotKey(None, hotkey_identifiers::SWAP_PREVIOUS as i32, MOD_ALT, 0x48);

    let _swap_next = RegisterHotKey(None, hotkey_identifiers::SWAP_NEXT as i32, MOD_ALT, 0x4C);
    
    let _variant_previous = RegisterHotKey(None, hotkey_identifiers::VARIANT_PREVIOUS as i32, MOD_ALT | MOD_SHIFT, 0x4A);

    let _variant_next = RegisterHotKey(None, hotkey_identifiers::VARIANT_NEXT as i32, MOD_ALT | MOD_SHIFT, 0x4B);

    let _layout_previous = RegisterHotKey(None, hotkey_identifiers::LAYOUT_PREVIOUS as i32, MOD_ALT | MOD_SHIFT, 0x48);

    let _layout_next = RegisterHotKey(None, hotkey_identifiers::LAYOUT_NEXT as i32, MOD_ALT | MOD_SHIFT, 0x4C);

    let _focus_previous_monitor = RegisterHotKey(None, hotkey_identifiers::FOCUS_PREVIOUS_MONITOR as i32, MOD_ALT, 0x55);

    let _focus_next_monitor = RegisterHotKey(None, hotkey_identifiers::FOCUS_NEXT_MONITOR as i32, MOD_ALT, 0x49);

    let _swap_previous_monitor = RegisterHotKey(None, hotkey_identifiers::SWAP_PREVIOUS_MONITOR as i32, MOD_ALT, 0x59);

    let _swap_next_monitor = RegisterHotKey(None, hotkey_identifiers::SWAP_NEXT_MONITOR as i32, MOD_ALT, 0x4F);
    
    let _grab_window = RegisterHotKey(None, hotkey_identifiers::GRAB_WINDOW as i32, MOD_ALT | MOD_SHIFT | MOD_NOREPEAT, 0x55);

    let _release_window = RegisterHotKey(None, hotkey_identifiers::RELEASE_WINDOW as i32, MOD_ALT | MOD_SHIFT | MOD_NOREPEAT, 0x49);

    let _refresh_workspace = RegisterHotKey(None, hotkey_identifiers::REFRESH_WORKSPACE as i32, MOD_ALT | MOD_SHIFT | MOD_NOREPEAT, 0x59);

    let _toggle_workspace = RegisterHotKey(None, hotkey_identifiers::TOGGLE_WORKSPACE as i32, MOD_ALT | MOD_SHIFT | MOD_NOREPEAT, 0x4F);

}

unsafe fn handle_message(msg: MSG, wm: &mut wm::WindowManager) {

    match msg.message {

        wm::messages::WINDOW_CREATED => {

            wm.window_created(HWND(msg.wParam.0 as *mut core::ffi::c_void));

        },

        wm::messages::WINDOW_DESTROYED => {

            wm.window_destroyed(HWND(msg.wParam.0 as *mut core::ffi::c_void));

        },

        wm::messages::WINDOW_MINIMIZED_OR_MAXIMIZED => {

            wm.window_minimized_or_maximized(HWND(msg.wParam.0 as *mut core::ffi::c_void));

        },

        wm::messages::WINDOW_CLOAKED => {

            wm.window_cloaked(HWND(msg.wParam.0 as *mut core::ffi::c_void));

        },

        wm::messages::FOREGROUND_WINDOW_CHANGED => {

            wm.foreground_window_changed(HWND(msg.wParam.0 as *mut core::ffi::c_void));

        },

        wm::messages::WINDOW_MOVE_FINISHED => {

            wm.window_move_finished(HWND(msg.wParam.0 as *mut core::ffi::c_void));

        },

        WM_HOTKEY => {

            match msg.wParam.0 {
                
                hotkey_identifiers::FOCUS_PREVIOUS => {

                    wm.focus_previous();

                },

                hotkey_identifiers::FOCUS_NEXT => {

                    wm.focus_next();

                },

                hotkey_identifiers::SWAP_PREVIOUS => {

                    wm.swap_previous();

                },

                hotkey_identifiers::SWAP_NEXT => {

                    wm.swap_next();

                },

                hotkey_identifiers::VARIANT_PREVIOUS => {

                    wm.variant_previous();

                },

                hotkey_identifiers::VARIANT_NEXT => {

                    wm.variant_next();

                },

                hotkey_identifiers::LAYOUT_PREVIOUS => {

                    wm.layout_previous();

                },

                hotkey_identifiers::LAYOUT_NEXT => {

                    wm.layout_next();

                },

                hotkey_identifiers::FOCUS_PREVIOUS_MONITOR => {

                    wm.focus_previous_monitor();

                },

                hotkey_identifiers::FOCUS_NEXT_MONITOR => {

                    wm.focus_next_monitor();

                },

                hotkey_identifiers::SWAP_PREVIOUS_MONITOR => {

                    wm.swap_previous_monitor();

                },

                hotkey_identifiers::SWAP_NEXT_MONITOR => {

                    wm.swap_next_monitor();

                },

                hotkey_identifiers::GRAB_WINDOW => {

                    wm.grab_window();

                },

                hotkey_identifiers::RELEASE_WINDOW => {

                    wm.release_window();

                },

                hotkey_identifiers::REFRESH_WORKSPACE => {

                    wm.refresh_workspace();

                },

                hotkey_identifiers::TOGGLE_WORKSPACE => {

                    wm.toggle_workspace();

                },

                _ => (),

            }

        },

        _ => (),
    
    }

}

fn main() {

    let mut msg = MSG::default();

    unsafe {

        register_hotkeys();

        let mut wm = wm::WindowManager::new();

        test::test(&mut wm);

        loop {
            
            GetMessageA(&mut msg, None, 0, 0);

            handle_message(msg, &mut wm);

        }

    }

}

mod test {
    
    pub unsafe fn test(wm: &mut crate::wm::WindowManager) {

            

            wm.initialize_monitors();

            let primary_hmonitor = wm.get_monitor_vec()[0];

            let mut layout_group = super::layout::LayoutGroup::new(primary_hmonitor);

            let mut idx = layout_group.default_idx();

            let layout = &mut layout_group.get_layouts_mut()[idx];

            let end_tiling_behaviour = super::layout::EndTilingBehaviour::default_repeating();


            layout.set_end_tiling_behaviour(end_tiling_behaviour);
            layout.new_zone_vec();
            layout.split(1, 0, super::layout::SplitDirection::Vertical(960));

            layout.add_repeating_split(super::layout::Direction::Horizontal, 0.5, 4, false);
            layout.add_repeating_split(super::layout::Direction::Vertical, 0.5, 1, true);
            layout.add_repeating_split(super::layout::Direction::Horizontal, 0.5, 2, true);
            layout.add_repeating_split(super::layout::Direction::Vertical, 0.5, 3, false);
            

            //layout.new_zone_vec();
            //layout.split(1, 0, super::layout::SplitDirection::Vertical(960));
            //layout.new_zone_vec_from(1);
            //layout.split(2, 1, super::layout::SplitDirection::Horizontal(600));
            //
            //layout.set_end_tiling_direction(super::layout::Direction::Vertical);
            
            

            layout_group.new_variant();

            idx = layout_group.layouts_len() - 1;

            let second_variant = &mut layout_group.get_layouts_mut()[idx];

            second_variant.merge_zones(1, 0, 1);

            second_variant.split(1, 0, super::layout::SplitDirection::Vertical(1280));
            layout_group.new_variant();
            idx = layout_group.layouts_len() - 1;

            let third_variant = &mut layout_group.get_layouts_mut()[idx];

            third_variant.merge_zones(1, 0, 1);

            third_variant.split(1, 0, super::layout::SplitDirection::Vertical(1440));
            layout_group.new_variant();
            idx = layout_group.layouts_len() - 1;

            let fourth_variant = &mut layout_group.get_layouts_mut()[idx];

            fourth_variant.merge_zones(1, 0, 1);

            fourth_variant.split(1, 0, super::layout::SplitDirection::Vertical(640));

            fourth_variant.swap_zones(1, 0, 1);

            layout_group.move_variant(idx, 0);


            let mut second_layout_group = super::layout::LayoutGroup::new(primary_hmonitor);

            idx = second_layout_group.default_idx();

            let l2 = &mut second_layout_group.get_layouts_mut()[idx];


            l2.set_end_tiling_direction(super::layout::Direction::Vertical);

            l2.set_end_tiling_start_from(3);

            l2.new_zone_vec();

            l2.split(1, 0, super::layout::SplitDirection::Vertical(960));

            l2.new_zone_vec_from(1);

            l2.split(2, 1, super::layout::SplitDirection::Horizontal(600));

            l2.new_zone_vec();

            l2.split(3, 0, super::layout::SplitDirection::Horizontal(600));

            wm.get_settings_mut().set_disable_rounding(true);
            wm.get_settings_mut().set_disable_unfocused_border(true);
            wm.get_settings_mut().set_window_padding(12);
            wm.get_settings_mut().set_edge_padding(24);

            wm.initialize_with_layout_group(layout_group);
            wm.add_layout_group(second_layout_group);

    }
}
