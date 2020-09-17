pub fn open_console() {
    #[cfg(windows)]
    {
        use winapi::um::consoleapi::AllocConsole;
        unsafe {
            AllocConsole();
        }
    }
}

pub fn close_console() {
    #[cfg(windows)]
    {
        use winapi::um::wincon::FreeConsole;
        unsafe {
            FreeConsole();
        }
    }
}
