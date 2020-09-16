#[cfg(windows)]
pub mod windows {}

#[cfg(windows)]
#[macro_export]
macro_rules! make_entrypoint {
    ($attach_fn:expr, $detach_fn:expr) => {
        use $crate::winapi::shared::minwindef::*;
        static INJECTION_ENTRYPOINT_THREAD_SENDER: OnceCell<Mutex<Sender<()>>> = OnceCell::new();

        #[no_mangle]
        #[allow(non_snake_case, unused_variables)]
        pub extern "system" fn DllMain(
            dll_module: HINSTANCE,
            call_reason: DWORD,
            reserved: LPVOID)
            -> BOOL {

            const DLL_PROCESS_ATTACH: DWORD = 1;
            const DLL_PROCESS_DETACH: DWORD = 0;

            match call_reason {
                DLL_PROCESS_ATTACH => {
                    let (tx, rx) = mpsc::channel();
                    INJECTION_ENTRYPOINT_THREAD_SENDER.set(Mutex::new(tx)).expect("Failed to set thread kill sender");
                    std::thread::spawn(move || {
                        $attach_fn();
                        rx.recv().expect("Failed to receive run thread kill command");
                    });
                },
                DLL_PROCESS_DETACH => {
                    let tx = INJECTION_ENTRYPOINT_THREAD_SENDER.get().expect("Failed to get run thread kill sender");
                    let tx = tx.lock().expect("Failed to acquire run thread kill sender");
                    tx.send(()).expect("Failed to send kill command to run thread");
                    $detach_fn();
                },
                _ => ()
            }

            return TRUE;
        }
    };
}

#[cfg(macos)]
pub mod macos {}

#[cfg(macos)]
#[macro_export]
macro_rules! make_entrypoint {
    ($attach_fn:expr, $detach_fn:expr) => {
        compile_error!("Entrypoints for MacOS are not supported at this time");
    };
}

#[cfg(linux)]
pub mod linux {}


#[cfg(linux)]
#[macro_export]
macro_rules! make_entrypoint {
    ($attach_fn:expr, $detach_fn:expr) => {
        compile_error!("Entrypoints for Linux are not supported at this time");
    };
}
