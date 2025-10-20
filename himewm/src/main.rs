use himewm::*;
use windows::Win32::UI::WindowsAndMessaging::*;

fn main() {
    let console_hwnd = util::get_console_hwnd();
    let _hide_console_window = windows_api::show_window(console_hwnd, SW_HIDE);
    if let Err(e) = directories::create_dirs() {
        match e.kind() {
            std::io::ErrorKind::AlreadyExists => (),
            _ => {
                util::display_message(
                    console_hwnd,
                    &util::MessageType::Error,
                    "Error: Failed to create himewm config directories",
                );
                windows_api::post_quit_message(0);
            }
        }
    }
    let tray_icon = tray_icon::create();
    if let Err(_) = tray_icon {
        util::display_message(
            console_hwnd,
            &util::MessageType::Error,
            "Error: Failed to create himewm tray icon",
        );
        windows_api::post_quit_message(0);
    }
    let mut window_manager: Option<wm::WindowManager> = None;
    tray_icon::set_menu_event_handler();
    let mut previous_keybinds = None;
    let mut msg = MSG::default();
    let mut first_iteration = true;
    while first_iteration || windows_api::get_message(&mut msg, None, 0, 0).as_bool() {
        first_iteration = false;
        let _translate_message = windows_api::translate_message(&msg);
        windows_api::dispatch_message(&msg);
        match &mut window_manager {
            Some(wm) if !wm.restart_requested() => {
                wm::message_handler::handle_message(msg, wm);
            }
            _ => {
                let user_config::UserConfig {
                    config:
                        user_config::Config {
                            settings,
                            window_rules,
                            layouts,
                            keybinds,
                        },
                    mut warnings,
                    errors,
                } = user_config::get_user_config();
                if let Some(registered_keybinds) = previous_keybinds {
                    keybinds::unregister_hotkeys(registered_keybinds, &mut warnings);
                }
                keybinds::register_hotkeys(&keybinds, &mut warnings);
                previous_keybinds = Some(keybinds);
                let mut message_type = util::MessageType::None;
                let mut message = String::new();
                if !warnings.is_empty() {
                    util::add_to_message(&mut message, &warnings);
                    message_type = util::MessageType::Warning;
                }
                if !errors.is_empty() {
                    util::add_to_message(&mut message, &errors);
                    message_type = util::MessageType::Error;
                }
                if !message.is_empty() {
                    util::display_message(console_hwnd, &message_type, &message);
                }
                match message_type {
                    util::MessageType::Error => {
                        windows_api::post_quit_message(0);
                    }
                    _ => {
                        let mut existing_event_hook = None;
                        let mut existing_vd_manager = None;
                        if let Some(wm) = window_manager {
                            existing_vd_manager = Some(wm.virtual_desktop_manager().to_owned());
                            existing_event_hook = Some(wm.event_hook());
                        }
                        window_manager = Some(wm::WindowManager::new(
                            settings,
                            window_rules,
                            existing_event_hook,
                            existing_vd_manager,
                        ));
                        if let Some(wm) = &mut window_manager {
                            wm.initialize(layouts);
                        }
                    }
                }
            }
        }
    }
    if let Some(wm) = window_manager {
        wm.exit();
    }
}
