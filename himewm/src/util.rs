use crate::windows_api;
use windows::Win32::System::Console::*;

pub fn show_error_message(message: &str) {
    let _free_console = windows_api::free_console();
    let _alloc_console = windows_api::alloc_console();
    let handle = windows_api::get_std_handle(STD_INPUT_HANDLE).unwrap();
    let mut console_mode = CONSOLE_MODE::default();
    let _get_console_mode = windows_api::get_console_mode(handle, &mut console_mode);
    let _set_console_mode =
        windows_api::set_console_mode(handle, console_mode & !ENABLE_ECHO_INPUT);
    println!("{}", message);
    println!("Press ENTER to exit");
    let mut buf = String::new();
    let _read_line = std::io::stdin().read_line(&mut buf);
}
