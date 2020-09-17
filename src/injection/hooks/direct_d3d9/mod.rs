use crate::injection::hooks::{Hook, InstallError, UninstallError};
use crate::injection::memory::pattern_scan;
use std::ffi::CString;
use winapi::ctypes::c_void;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::HRESULT;
use winapi::um::consoleapi::AllocConsole;
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
use winapi::um::memoryapi::VirtualProtect;
use winapi::um::winnt::*;

const MODULE_NAME: &'static str = "d3d9.dll";

#[derive(Debug)]
pub enum DirectD3D9InstallError {
    ModuleNotFound,
    VirtualProtectVMT,
    VirtualProtectVMTRestore,
}

#[derive(Debug)]
pub enum DirectD3D9UninstallError {
    Other,
}

type EndSceneFn = unsafe extern "C" fn() -> HRESULT;

#[no_mangle]
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

impl Hook<DirectD3D9InstallError, DirectD3D9UninstallError> for DirectD3D9 {
    fn is_installed(&self) -> bool {
        self.installed
    }

    fn install(&mut self) -> Result<(), InstallError<DirectD3D9InstallError>> {
        if self.installed {
            return Err(InstallError::AlreadyInstalled);
        }

        let module_name =
            CString::new(MODULE_NAME).expect("Could not turn MODULE_NAME into a C string");
        let module = unsafe { GetModuleHandleA(module_name.as_ptr()) } as *const u8;
        if module == std::ptr::null() {
            return Err(InstallError::Custom(DirectD3D9InstallError::ModuleNotFound));
        }
        println!("Module handle d3d9.dll addr: {:p}", module);

        let vmt_addr = unsafe {
            pattern_scan(
                module,
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
            .expect("Pattern scan for d3d9 vmt failed")
            .offset(2)
        };

        // EndScene
        {
            let end_scene_addr = unsafe { vmt_addr.offset(42) };
            let end_scene = unsafe { std::mem::transmute::<_, EndSceneFn>(end_scene_addr) };
            println!("End Scene address: {:p}", end_scene_addr);

            let mut old_protection: DWORD = 0;
            // Get access to write vmt memory area
            {
                let success = unsafe {
                    VirtualProtect(
                        end_scene_addr as *mut c_void,
                        4,
                        PAGE_EXECUTE_READWRITE,
                        &mut old_protection,
                    )
                };
                if success == 0 {
                    return Err(InstallError::Custom(
                        DirectD3D9InstallError::VirtualProtectVMT,
                    ));
                }
            }

            unsafe {
                *(end_scene_addr as *mut u32) = std::mem::transmute::<_, u32>(&end_scene_hook);
            }

            // Restore previous protection for vmt memory area
            {
                let success = unsafe {
                    VirtualProtect(
                        end_scene_addr as *mut c_void,
                        4,
                        old_protection,
                        &mut old_protection,
                    )
                };
                if success == 0 {
                    return Err(InstallError::Custom(
                        DirectD3D9InstallError::VirtualProtectVMTRestore,
                    ));
                }
            }
        }

        self.installed = true;
        Ok(())
    }

    fn uninstall(&mut self) -> Result<(), UninstallError<DirectD3D9UninstallError>> {
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
