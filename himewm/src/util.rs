use crate::{directories, windows_api};
use serde::{Deserialize, Serialize};
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

pub fn initialize_user_config<T>(file_name: &str) -> T where for<'a> T: Default + Deserialize<'a> + Serialize {
    let dirs = directories::Directories::new();
    let config_path = dirs.config_dir.join(format!("{file_name}"));
    match std::fs::read(&config_path) {
        Ok(byte_vector) => match serde_json::from_slice::<T>(byte_vector.as_slice()) {
            Ok(user_config) => {
                return user_config;
            }
            Err(_) => {
                return T::default();
            }
        },
        Err(_) => {
            let file = std::fs::File::create_new(config_path).unwrap();
            let default_user_config = T::default();
            let _ = serde_json::to_writer_pretty(&file, &default_user_config);
            return default_user_config;
        }
    }
}
