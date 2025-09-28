use crate::{windows_api, wm_messages};
use tray_icon::{
    menu::{Menu, MenuEvent, MenuId, MenuItemBuilder},
    Icon, TrayIcon, TrayIconBuilder,
};
use windows::Win32::Foundation::{LPARAM, WPARAM};

fn get_icon() -> Icon {
    let icon_bytes = include_bytes!("../assets/icon.data");
    let rgba = Vec::from(icon_bytes);
    return Icon::from_rgba(rgba, 256, 256).unwrap();
}

pub fn create() -> tray_icon::Result<TrayIcon> {
    let menu = Menu::new();
    let restart_item = MenuItemBuilder::new()
        .id(MenuId::new(wm_messages::tray_menu_ids::RESTART))
        .text("Restart himewm")
        .enabled(true)
        .build();
    let quit_item = MenuItemBuilder::new()
        .id(MenuId::new(wm_messages::tray_menu_ids::QUIT))
        .text("Quit")
        .enabled(true)
        .build();
    menu.append(&restart_item).unwrap();
    menu.append(&quit_item).unwrap();
    let tooltip = format!("himewm v{}", env!("CARGO_PKG_VERSION"));
    let icon = get_icon();
    return TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(tooltip)
        .with_icon(icon)
        .build();
}

pub fn set_menu_event_handler() {
    MenuEvent::set_event_handler(Some(|event: MenuEvent| match event.id().as_ref() {
        wm_messages::tray_menu_ids::QUIT => {
            windows_api::post_quit_message(0);
        }
        wm_messages::tray_menu_ids::RESTART => {
            windows_api::post_message(
                None,
                wm_messages::messages::REQUEST_RESTART,
                WPARAM::default(),
                LPARAM::default(),
            )
            .unwrap();
        }
        _ => return,
    }));
}
