#![windows_subsystem = "windows"]
use himewm::*;
use windows::Win32::UI::WindowsAndMessaging::*;

fn main() {
    // Maybe error handle this
    let _create_dirs = directories::create_dirs();
    let user_settings =
        util::initialize_user_config::<user_settings::UserSettings>("settings.json");
    let user_window_rules = util::initialize_user_config("window_rules.json");
    let mut msg = MSG::default();
    let layouts = match layouts::initialize_layouts() {
        Some(val) => val,
        None => {
            util::show_error_message("No layouts found");
            return;
        }
    };
    let layout_idx_map = layouts::get_layout_idx_map(&layouts);
    let internal_window_rules =
        window_rules::get_internal_window_rules(&user_window_rules, &layout_idx_map);
    keybinds::register_hotkeys();
    let _create_tray_icon = tray_menu::create();
    tray_menu::set_menu_event_handler();
    let mut wm = wm::WindowManager::new(
        user_settings.to_settings(&layout_idx_map),
        internal_window_rules,
    );
    wm.initialize(layouts.into_iter().map(|(_, layout)| layout).collect());
    while windows_api::get_message(&mut msg, None, 0, 0).as_bool() {
        wm_message_handler::handle_message(msg, &mut wm);
        let _translate_message = windows_api::translate_message(&msg);
        windows_api::dispatch_message(&msg);
    }
    let _unhook_win_event = windows_api::unhook_win_event(wm.get_event_hook());
    windows_api::co_uninitialize();
}
