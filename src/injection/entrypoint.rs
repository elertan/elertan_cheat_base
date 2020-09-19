use std::error::Error;
use std::fmt::{Debug, Display};

#[derive(Debug, thiserror::Error)]
pub enum AttachError<T: 'static + Debug + Display + Error> {
    #[error(transparent)]
    Custom(T),
}

#[derive(Debug, thiserror::Error)]
pub enum DetachError<T: 'static + Debug + Display + Error> {
    #[error(transparent)]
    Custom(T),
}

pub trait Entrypoint<A, D>
where
    A: 'static + Debug + Display + Error,
    D: 'static + Debug + Display + Error,
{
    fn attach(&mut self) -> Result<(), AttachError<A>>;
    fn detach(&mut self) -> Result<(), DetachError<D>>;
}

#[cfg(windows)]
#[macro_export]
macro_rules! generate_entrypoint {
    ($mutex_entrypoint:expr) => {
        use $crate::winapi::shared::minwindef::*;

        #[no_mangle]
        #[allow(non_snake_case, unused_variables)]
        pub extern "system" fn DllMain(
            dll_module: HINSTANCE,
            call_reason: DWORD,
            reserved: LPVOID,
        ) -> BOOL {
            use std::sync::mpsc::Sender;
            use std::sync::{mpsc, Mutex};
            use $crate::log::trace;
            use $crate::once_cell::sync::OnceCell;

            static __ELERTAN_CHEATBASE_ENTRYPOINT_SENDER: OnceCell<Mutex<Sender<()>>> =
                OnceCell::new();

            const DLL_PROCESS_ATTACH: DWORD = 1;
            const DLL_PROCESS_DETACH: DWORD = 0;

            match call_reason {
                DLL_PROCESS_ATTACH => {
                    $crate::log_panics::init();
                    trace!("DllMain -> DLL_PROCESS_ATTACH");
                    let (tx, rx) = mpsc::channel();

                    __ELERTAN_CHEATBASE_ENTRYPOINT_SENDER
                        .set(Mutex::new(tx))
                        .expect("Failed to set __ELERTAN_CHEATBASE_ENTRYPOINT_SENDER");
                    std::thread::spawn(move || {
                        $crate::log_panics::init();
                        let mut entrypoint = $mutex_entrypoint
                            .lock()
                            .expect("Failed to lock mutex entrypoint");
                        entrypoint.attach().expect("Attach failed");
                        rx.recv().unwrap();
                    });
                }
                DLL_PROCESS_DETACH => {
                    trace!("DllMain -> DLL_PROCESS_DETACH");
                    let tx = __ELERTAN_CHEATBASE_ENTRYPOINT_SENDER
                        .get()
                        .expect("Failed to get  __ELERTAN_CHEATBASE_ENTRYPOINT_SENDER");
                    trace!("Got entrypoint sender");
                    let tx = tx
                        .lock()
                        .expect("Failed to acquire lock on  __ELERTAN_CHEATBASE_ENTRYPOINT_SENDER");
                    tx.send(()).expect("Could not send release");
                    let mut entrypoint = $mutex_entrypoint
                        .lock()
                        .expect("Failed to lock mutex entrypoint");
                    entrypoint.detach().expect("Detach failed");
                }
                _ => (),
            }

            return TRUE;
        }
    };
}

#[cfg(macos)]
#[macro_export]
macro_rules! generate_entrypoint {
    ($mutex_entrypoint:expr) => {
        compile_error!("Entrypoints for MacOS are not supported at this time");
    };
}

#[cfg(linux)]
#[macro_export]
macro_rules! generate_entrypoint {
    ($mutex_entrypoint:expr) => {
        compile_error!("Entrypoints for Linux are not supported at this time");
    };
}
