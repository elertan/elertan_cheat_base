use crate::injection::hooks::{Hook, InstallError, UninstallError};
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::HRESULT;
use std::ffi::{CString};

const MODULE_NAME: &'static str = "d3d9.dll";
const END_SCENE_NAME: &'static str = "EndScene";

type EndSceneFn = extern fn() -> HRESULT;

extern fn end_scene_hook() -> HRESULT {
    0
}

pub struct DirectD3D9 {
    installed: bool,
}

impl DirectD3D9 {
    pub fn new() -> Self {
        Self {
            installed: false,
        }
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

        let module_name = CString::new(MODULE_NAME).expect("Could not turn MODULE_NAME into a C string");
        let module = unsafe { GetModuleHandleA(module_name.as_ptr()) };

        // EndScene
        {
            let end_scene_name = CString::new(END_SCENE_NAME).expect("Could not turn END_SCENE_NAME into a C string");
            let proc_address = unsafe { GetProcAddress(module, end_scene_name.as_ptr()) };

            let end_scene = unsafe { std::mem::transmute::<*mut __some_function, EndSceneFn>(proc_address) };
        }

        self.installed = true;
        Ok(())
    }

    fn uninstall(&mut self) -> Result<(), UninstallError> {
        if self.installed {
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
