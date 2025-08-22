use windows::{
    core::PSTR, 
    Win32::{
        Foundation::*, 
        System::{
            Console::*,
            Threading::*,
        }, 
        UI::WindowsAndMessaging::*,
    }
};

const MAX_PATH_LEN: usize = 1024;

pub unsafe fn show_error_message(message: &str) {
    let _free_console = FreeConsole();
    let _alloc_console = AllocConsole();
    let handle = GetStdHandle(STD_INPUT_HANDLE).unwrap();
    let mut console_mode = CONSOLE_MODE::default();
    let _get_console_mode = GetConsoleMode(handle, &mut console_mode);
    let _set_console_mode = SetConsoleMode(handle, console_mode & !ENABLE_ECHO_INPUT);
    println!("{}", message);
    println!("Press ENTER to exit");
    let mut buf = String::new();
    let _read_line = std::io::stdin().read_line(&mut buf);
}

pub unsafe fn get_window_name(hwnd: HWND) -> Result<String, std::string::FromUtf8Error> {
    let len = GetWindowTextLengthA(hwnd) as usize;
    let mut buf = vec![0 as u8; len + 1];
    GetWindowTextA(hwnd, &mut buf);
    return String::from_utf8(buf);
}

pub unsafe fn get_exe_name(hwnd: HWND) -> Result<String, std::string::FromUtf8Error> {
    let mut id = 0;
    GetWindowThreadProcessId(hwnd, Some(&mut id));
    let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, id).unwrap();
    let mut buf = [0 as u8; MAX_PATH_LEN];
    let mut size = MAX_PATH_LEN as u32;
    let _query = QueryFullProcessImageNameA(handle, PROCESS_NAME_FORMAT(0), PSTR(&mut buf as *mut u8), &mut size);
    let _close_handle = CloseHandle(handle);
    let path_string = String::from_utf8(Vec::from(&buf[0..size as usize]))?;
    let path = std::path::Path::new(&path_string);
    return Ok(String::from(path.file_name().unwrap().to_str().unwrap()));
}
