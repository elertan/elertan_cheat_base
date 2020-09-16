#[cfg(windows)]
pub mod windows {}

#[cfg(windows)]
#[macro_export]
macro_rules! make_entrypoint {
    ($attach_fn:expr, $detach_fn:expr) => {
        use $crate::winapi::shared::minwindef::*;

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
                DLL_PROCESS_ATTACH => $attach_fn(),
                DLL_PROCESS_DETACH => $detach_fn(),
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
