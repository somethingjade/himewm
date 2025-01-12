mod layout;
mod wm;

mod hotkey_identifiers {

    pub const FOCUS_PREVIOUS: usize = 0;

    pub const FOCUS_NEXT: usize = 1;

    pub const SWAP_PREVIOUS: usize = 2;

    pub const SWAP_NEXT: usize = 3;

    pub const VARIANT_PREVIOUS: usize = 6;

    pub const VARIANT_NEXT: usize = 7;

    pub const LAYOUT_PREVIOUS: usize = 4;

    pub const LAYOUT_NEXT: usize = 5;

}

unsafe fn register_hotkeys() {
    
    let _focus_previous = windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey(None, hotkey_identifiers::FOCUS_PREVIOUS as i32, windows::Win32::UI::Input::KeyboardAndMouse::MOD_ALT, 0x4A);

    let _focus_next = windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey(None, hotkey_identifiers::FOCUS_NEXT as i32, windows::Win32::UI::Input::KeyboardAndMouse::MOD_ALT, 0x4B);

    let _swap_previous = windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey(None, hotkey_identifiers::SWAP_PREVIOUS as i32, windows::Win32::UI::Input::KeyboardAndMouse::MOD_ALT, 0x48);

    let _swap_next = windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey(None, hotkey_identifiers::SWAP_NEXT as i32, windows::Win32::UI::Input::KeyboardAndMouse::MOD_ALT, 0x4C);
    
    let _variant_previous = windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey(None, hotkey_identifiers::VARIANT_PREVIOUS as i32, windows::Win32::UI::Input::KeyboardAndMouse::MOD_ALT | windows::Win32::UI::Input::KeyboardAndMouse::MOD_SHIFT, 0x4A);

    let _variant_next = windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey(None, hotkey_identifiers::VARIANT_NEXT as i32, windows::Win32::UI::Input::KeyboardAndMouse::MOD_ALT | windows::Win32::UI::Input::KeyboardAndMouse::MOD_SHIFT, 0x4B);

    let _layout_previous = windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey(None, hotkey_identifiers::LAYOUT_PREVIOUS as i32, windows::Win32::UI::Input::KeyboardAndMouse::MOD_ALT | windows::Win32::UI::Input::KeyboardAndMouse::MOD_SHIFT, 0x48);

    let _layout_next = windows::Win32::UI::Input::KeyboardAndMouse::RegisterHotKey(None, hotkey_identifiers::LAYOUT_NEXT as i32, windows::Win32::UI::Input::KeyboardAndMouse::MOD_ALT | windows::Win32::UI::Input::KeyboardAndMouse::MOD_SHIFT, 0x4C);

}

unsafe fn handle_message(msg: windows::Win32::UI::WindowsAndMessaging::MSG, wm: &mut wm::WindowManager) {

    match msg.message {

        wm::messages::WINDOW_CREATED => {

            wm.window_created(windows::Win32::Foundation::HWND(msg.wParam.0 as *mut core::ffi::c_void));

        },

        wm::messages::WINDOW_DESTROYED => {

            wm.window_destroyed(windows::Win32::Foundation::HWND(msg.wParam.0 as *mut core::ffi::c_void));

        },

        wm::messages::WINDOW_MINIMIZED_OR_MAXIMIZED => {

            wm.window_minimized_or_maximized(windows::Win32::Foundation::HWND(msg.wParam.0 as *mut core::ffi::c_void));

        },

        wm::messages::WINDOW_CLOAKED => {

            wm.window_cloaked(windows::Win32::Foundation::HWND(msg.wParam.0 as *mut core::ffi::c_void));

        },

        wm::messages::FOREGROUND_WINDOW_CHANGED => {

            wm.foreground_window_changed(windows::Win32::Foundation::HWND(msg.wParam.0 as *mut core::ffi::c_void));

        },

        wm::messages::WINDOW_MOVE_FINISHED => {

            wm.window_move_finished(windows::Win32::Foundation::HWND(msg.wParam.0 as *mut core::ffi::c_void));

        },

        windows::Win32::UI::WindowsAndMessaging::WM_HOTKEY => {

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

                _ => (),

            }

        },

        _ => (),
    
    }

}

fn main() {

    let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();

    unsafe {

        register_hotkeys();

        let mut wm = wm::WindowManager::new();

        test::test(&mut wm);

        loop {
            
            windows::Win32::UI::WindowsAndMessaging::GetMessageA(&mut msg, None, 0, 0);

            handle_message(msg, &mut wm);

        }

    }

}

mod test {

    fn display(layout: &super::layout::Layout) {

        for zones in &layout.zones {
            
            for zone in zones {

                println!("{zone:?}");

            }

            println!();

        }

    }
    
    pub unsafe fn test(wm: &mut crate::wm::WindowManager) {

            let mut layout_group = super::layout::LayoutGroup::new(windows::Win32::Graphics::Gdi::MonitorFromWindow(None, windows::Win32::Graphics::Gdi::MONITOR_DEFAULTTOPRIMARY));

            let mut idx = layout_group.default_idx();

            let layout = &mut layout_group.get_layouts_mut()[idx];

            layout.set_padding(4);
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
            
            
            display(&layout);

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


            let mut second_layout_group = super::layout::LayoutGroup::new(windows::Win32::Graphics::Gdi::MonitorFromWindow(None, windows::Win32::Graphics::Gdi::MONITOR_DEFAULTTOPRIMARY));

            idx = second_layout_group.default_idx();

            let l2 = &mut second_layout_group.get_layouts_mut()[idx];

            l2.set_padding(4);

            l2.set_end_tiling_direction(super::layout::Direction::Vertical);

            l2.set_end_tiling_start_from(3);

            l2.new_zone_vec();

            l2.split(1, 0, super::layout::SplitDirection::Vertical(960));

            l2.new_zone_vec_from(1);

            l2.split(2, 1, super::layout::SplitDirection::Horizontal(600));

            l2.new_zone_vec();

            l2.split(3, 0, super::layout::SplitDirection::Horizontal(600));

            wm.get_window_settings_mut().set_disable_rounding(true);
            wm.get_window_settings_mut().set_disable_unfocused_border(true);

            wm.initialize_monitors();

            wm.initialize_with_layout_group(layout_group);
            wm.add_layout_group(second_layout_group);

    }
}
