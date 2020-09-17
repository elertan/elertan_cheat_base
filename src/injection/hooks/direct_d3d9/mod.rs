use crate::injection::hooks::{Hook, InstallError, UninstallError};
use crate::injection::memory::pattern_scan;
use std::ffi::CString;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::HRESULT;
use winapi::um::consoleapi::AllocConsole;
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};

const MODULE_NAME: &'static str = "d3d9.dll";

type EndSceneFn = unsafe extern "C" fn() -> HRESULT;

extern "C" fn end_scene_hook() -> HRESULT {
    0
}

pub struct DirectD3D9 {
    installed: bool,
}

impl DirectD3D9 {
    pub fn new() -> Self {
        Self { installed: false }
    }
}

impl Hook for DirectD3D9 {
    fn is_installed(&self) -> bool {
        self.installed
    }

    fn install(&mut self) -> Result<(), InstallError> {
        if self.installed {
            return Err(InstallError::AlreadyInstalled);
        }

        let module_name =
            CString::new(MODULE_NAME).expect("Could not turn MODULE_NAME into a C string");
        let module = unsafe { GetModuleHandleA(module_name.as_ptr()) };
        println!("Module handle d3d9.dll addr: {:p}", module);

        // EndScene
        {
            let end_scene_addr = unsafe {
                pattern_scan(
                    module as *const u8,
                    0x128000,
                    &[
                        Some(0xC7),
                        Some(0x06),
                        None,
                        None,
                        None,
                        None,
                        Some(0x89),
                        Some(0x86),
                        None,
                        None,
                        None,
                        None,
                        Some(0x89),
                        Some(0x86),
                    ],
                )
            }
            .expect("Pattern scan for end scene failed");

            let end_scene = unsafe { std::mem::transmute::<_, EndSceneFn>(end_scene_addr) };
            println!("End Scene address: {:p}", end_scene_addr);
        }

        self.installed = true;
        Ok(())
    }

    fn uninstall(&mut self) -> Result<(), UninstallError> {
        if !self.installed {
            return Err(UninstallError::NotInstalled);
        }
        self.installed = false;
        Ok(())
    }
}

impl Drop for DirectD3D9 {
    fn drop(&mut self) {
        if !self.installed {
            return;
        }
        self.uninstall().expect("Could not uninstall");
    }
}
