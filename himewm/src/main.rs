#![windows_subsystem = "windows"]
use windows::Win32::{
    System::Com::*,
    UI::{Accessibility::*, WindowsAndMessaging::*},
};
use himewm::*;

fn main() {
    // Maybe error handle this
    let _create_dirs = init::create_dirs();
    let user_settings = init::initialize_settings();
    let mut msg = MSG::default();
    unsafe {
        let layouts = match init::initialize_layouts() {
            Some(val) => val,
            None => {
                wm::show_error_message("No layouts found");
                return;
            }
        };
        wm::register_hotkeys();
        let _create_tray_icon = tray_menu::create();
        tray_menu::set_menu_event_handler();
        let mut wm = wm::WindowManager::new(user_settings.to_settings(&layouts));
        wm.initialize(layouts.into_iter().map(|(_, layout)| layout).collect());
        while GetMessageA(&mut msg, None, 0, 0).as_bool() {
            wm::handle_message(msg, &mut wm);
            let _translate_message = TranslateMessage(&msg);
            DispatchMessageA(&msg);
        }
        let _unhook_win_event = UnhookWinEvent(wm.get_event_hook());
        CoUninitialize();
    }
}
