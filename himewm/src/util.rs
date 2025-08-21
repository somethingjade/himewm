use windows::Win32::System::Console::*;

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
