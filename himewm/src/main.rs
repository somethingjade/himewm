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
                let mut message_type = util::MessageType::Warning;
                let mut message = String::new();
                util::add_to_message(&mut message, &warnings);
                if !errors.is_empty() {
                    util::add_to_message(&mut message, &errors);
                    message_type = util::MessageType::Error;
                }
                if !message.is_empty() {
                    util::display_message(console_hwnd, &message_type, &message);
                    if let util::MessageType::Error = message_type {
                        windows_api::post_quit_message(0);
                    }
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
        let _translate_message = windows_api::translate_message(&msg);
        windows_api::dispatch_message(&msg);
    }
    if let Some(wm) = window_manager {
        wm.exit();
    }
}
