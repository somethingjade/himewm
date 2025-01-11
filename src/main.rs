mod layout;
mod wm;

fn main() {

    let mut msg = windows::Win32::UI::WindowsAndMessaging::MSG::default();

    unsafe {

        let mut wm = wm::WindowManager::new();

        test::test(&mut wm);

        loop {
            
            windows::Win32::UI::WindowsAndMessaging::GetMessageA(&mut msg, None, 0, 0);

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

                _ => continue,
            
            }

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

            let mut layout = super::layout::Layout::new(windows::Win32::Graphics::Gdi::MonitorFromWindow(None, windows::Win32::Graphics::Gdi::MONITOR_DEFAULTTOPRIMARY));

            layout.set_padding(4);
            let end_tiling_behaviour = super::layout::EndTilingBehaviour::default_repeating();

            layout.set_end_tiling_behaviour(end_tiling_behaviour);

            layout.add_repeating_split(super::layout::Direction::Vertical, 0.5, 4, false);
            layout.add_repeating_split(super::layout::Direction::Horizontal, 0.5, 1, false);
            layout.add_repeating_split(super::layout::Direction::Vertical, 0.5, 2, true);
            layout.add_repeating_split(super::layout::Direction::Horizontal, 0.5, 3, true);
            
            //layout.new_zone_vec();
            //layout.split(1, 0, super::layout::SplitDirection::Vertical(960));
            //layout.new_zone_vec_from(1);
            //layout.split(2, 1, super::layout::SplitDirection::Horizontal(600));
            //
            //layout.set_end_tiling_direction(super::layout::Direction::Vertical);
            
            layout.update();
            
            display(&layout);


            wm.get_window_settings_mut().set_disable_rounding(true);
            wm.get_window_settings_mut().set_disable_unfocused_border(true);

            wm.initialize_monitors();

            for h in wm.hmonitor_default_layout_indices.keys() {

                let mut monitor_info = windows::Win32::Graphics::Gdi::MONITORINFO::default();

                monitor_info.cbSize = std::mem::size_of::<windows::Win32::Graphics::Gdi::MONITORINFO>() as u32;

                windows::Win32::Graphics::Gdi::GetMonitorInfoA(windows::Win32::Graphics::Gdi::HMONITOR(*h), &mut monitor_info).unwrap();

                println!("{:?}, {:?}", h, monitor_info.rcWork);

            }

            wm.initialize_with_layout(layout);

    }
}
