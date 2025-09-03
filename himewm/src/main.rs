#![windows_subsystem = "windows"]
use himewm::*;
use windows::Win32::UI::WindowsAndMessaging::*;

fn main() {
    // Maybe error handle this
    let _create_dirs = directories::create_dirs();
    let mut window_manager: Option<wm::WindowManager> = None;
    let _create_tray_icon = tray_menu::create();
    tray_menu::set_menu_event_handler();
    let mut msg = MSG::default();
    let mut first_iteration = true;
    while first_iteration || windows_api::get_message(&mut msg, None, 0, 0).as_bool() {
        first_iteration = false;
        match &mut window_manager {
            Some(wm) if !wm.restart_requested() => {
                wm_message_handler::handle_message(msg, wm);
            }
            _ => {
                let mut existing_event_hook = None;
                let mut existing_vd_manager = None;
                if let Some(wm) = window_manager {
                    existing_vd_manager = Some(wm.get_virtual_desktop_manager().to_owned());
                    existing_event_hook = Some(wm.get_event_hook());
                }
                let user_settings =
                    util::initialize_user_config::<user_settings::UserSettings>("settings.json");
                let user_window_rules = util::initialize_user_config("window_rules.json");
                let user_keybinds = util::initialize_user_config("keybinds.json");
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
                let internal_keybinds = keybinds::Keybinds::from(&user_keybinds);
                keybinds::register_hotkeys(internal_keybinds);
                window_manager = Some(wm::WindowManager::new(
                    user_settings.to_settings(&layout_idx_map),
                    internal_window_rules,
                    existing_event_hook,
                    existing_vd_manager,
                ));
                if let Some(wm) = &mut window_manager {
                    wm.initialize(layouts.into_iter().map(|(_, layout)| layout).collect());
                }
            }
        }
        let _translate_message = windows_api::translate_message(&msg);
        windows_api::dispatch_message(&msg);
    }
    if let Some(wm) = window_manager {
        wm.exit();
    }
}
