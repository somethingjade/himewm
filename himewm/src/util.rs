use crate::windows_api;
use windows::{
    core::PSTR,
    Win32::{
        Foundation::*,
        System::{
            Console::*, 
            Threading::*
        },
    },
};

const MAX_PATH_LEN: usize = 1024;

pub fn show_error_message(message: &str) {
    let _free_console = windows_api::free_console();
    let _alloc_console = windows_api::alloc_console();
    let handle = windows_api::get_std_handle(STD_INPUT_HANDLE).unwrap();
    let mut console_mode = CONSOLE_MODE::default();
    let _get_console_mode = windows_api::get_console_mode(handle, &mut console_mode);
    let _set_console_mode = windows_api::set_console_mode(handle, console_mode & !ENABLE_ECHO_INPUT);
    println!("{}", message);
    println!("Press ENTER to exit");
    let mut buf = String::new();
    let _read_line = std::io::stdin().read_line(&mut buf);
}

pub fn get_window_title(hwnd: HWND) -> Result<String, std::string::FromUtf8Error> {
    let len = windows_api::get_window_text_length(hwnd) as usize;
    let mut buf = vec![0 as u8; len + 1];
    windows_api::get_window_text(hwnd, &mut buf);
    return String::from_utf8(buf);
}

pub fn get_exe_name(hwnd: HWND) -> Result<String, std::string::FromUtf8Error> {
    let mut id = 0;
    windows_api::get_window_thread_process_id(hwnd, Some(&mut id));
    let handle = windows_api::open_process(PROCESS_QUERY_LIMITED_INFORMATION, false, id).unwrap();
    let mut buf = [0 as u8; MAX_PATH_LEN];
    let mut size = MAX_PATH_LEN as u32;
    let _query = windows_api::query_full_process_image_name(
        handle,
        PROCESS_NAME_FORMAT(0),
        PSTR(&mut buf as *mut u8),
        &mut size,
    );
    let _close_handle = windows_api::close_handle(handle);
    let path_string = String::from_utf8(Vec::from(&buf[0..size as usize]))?;
    let path = std::path::Path::new(&path_string);
    return Ok(String::from(path.file_name().unwrap().to_str().unwrap()));
}
