use crate::injection::hooks::{Hook, InstallError, UninstallError};
use crate::injection::memory::pattern_scan;
use std::ffi::CString;
use winapi::ctypes::c_void;
use winapi::shared::d3d9::*;
use winapi::shared::d3d9types::*;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::HRESULT;
use winapi::shared::windef::*;
use winapi::um::consoleapi::AllocConsole;
use winapi::um::libloaderapi::{GetModuleHandleA, GetProcAddress};
use winapi::um::memoryapi::VirtualProtect;
use winapi::um::winnt::*;
use winapi::um::winuser::{
    CreateWindowExA, DefWindowProcA, GetDesktopWindow, RegisterClassExA, CS_CLASSDC, WNDCLASSEXA,
    WS_EX_NOACTIVATE, WS_EX_OVERLAPPEDWINDOW, WS_EX_TRANSPARENT,
};

const MODULE_NAME: &'static str = "d3d9.dll";

#[derive(Debug)]
pub enum D3D9HookInstallError {
    ModuleNotFound,
    VirtualProtectVMT,
    VirtualProtectVMTRestore,
}

#[derive(Debug)]
pub enum D3D9HookUninstallError {
    Other,
}

type Direct3D9Create9Fn = unsafe extern "system" fn(version: UINT) -> *mut IDirect3D9;
type EndSceneFn = unsafe extern "system" fn() -> HRESULT;

#[no_mangle]
extern "system" fn end_scene_hook() -> HRESULT {
    0
}

#[no_mangle]
unsafe extern "system" fn tmp_wnd_proc(
    hwnd: HWND,
    uint: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    DefWindowProcA(hwnd, uint, wparam, lparam)
}

pub struct D3D9Hook {
    installed: bool,
}

impl D3D9Hook {
    pub fn new() -> Self {
        Self { installed: false }
    }
}

impl Hook<D3D9HookInstallError, D3D9HookUninstallError> for D3D9Hook {
    fn is_installed(&self) -> bool {
        self.installed
    }

    fn install(&mut self) -> Result<(), InstallError<D3D9HookInstallError>> {
        if self.installed {
            return Err(InstallError::AlreadyInstalled);
        }

        let module_name =
            CString::new(MODULE_NAME).expect("Could not turn MODULE_NAME into a C string");
        let module = unsafe { GetModuleHandleA(module_name.as_ptr()) } as *mut HINSTANCE__;
        if module == std::ptr::null_mut() {
            return Err(InstallError::Custom(D3D9HookInstallError::ModuleNotFound));
        }
        println!("Module handle d3d9.dll addr: {:p}", module);

        let d3d9_device = unsafe {
            let x = CString::new("Direct3DCreate9").unwrap();
            let direct3d_create_9_addr =
                std::mem::transmute::<_, Direct3D9Create9Fn>(GetProcAddress(module, x.as_ptr()));
            let d3d9_ctx = (direct3d_create_9_addr)(D3D_SDK_VERSION);
            println!("Created d3d9 ctx: {:?}", d3d9_ctx);
            let mut display_mode: D3DDISPLAYMODE = std::mem::zeroed();
            let d3d9_ctx_ref = d3d9_ctx.as_ref().unwrap();
            d3d9_ctx_ref.GetAdapterDisplayMode(D3DADAPTER_DEFAULT, &mut display_mode);
            println!("Filled display_mode: {}", display_mode.Width);
            let mut present_parameters: D3DPRESENT_PARAMETERS = std::mem::zeroed();
            present_parameters.Windowed = 1;
            present_parameters.SwapEffect = D3DSWAPEFFECT_DISCARD;
            present_parameters.BackBufferFormat = unsafe { display_mode.Format };
            println!("Got present_parameters");

            let wc_classname = CString::new("1").expect("Failed");
            let mut wc: WNDCLASSEXA = WNDCLASSEXA {
                cbSize: std::mem::size_of::<WNDCLASSEXA>() as u32,
                style: CS_CLASSDC,
                lpfnWndProc: Some(tmp_wnd_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: GetModuleHandleA(std::ptr::null()),
                hIcon: std::ptr::null::<()>() as *mut HICON__,
                hCursor: std::ptr::null::<()>() as *mut HICON__,
                hbrBackground: std::ptr::null::<()>() as *mut HBRUSH__,
                lpszMenuName: std::ptr::null::<()>() as *mut i8,
                lpszClassName: wc_classname.as_ptr(),
                hIconSm: std::ptr::null::<()>() as *mut HICON__,
            };
            RegisterClassExA(&mut wc);
            println!("Registered class");
            let hwnd = CreateWindowExA(
                0,
                wc_classname.as_ptr(),
                std::ptr::null::<i8>(),
                WS_EX_OVERLAPPEDWINDOW,
                100,
                100,
                300,
                300,
                GetDesktopWindow(),
                std::ptr::null::<()>() as *mut HMENU__,
                wc.hInstance,
                std::ptr::null::<()>() as *mut c_void,
            );
            println!("Created window: {}", std::mem::transmute::<_, u32>(hwnd));

            let mut device: *mut IDirect3DDevice9 = std::ptr::null_mut();
            let h_result = d3d9_ctx_ref.CreateDevice(
                D3DADAPTER_DEFAULT,
                D3DDEVTYPE_HAL,
                hwnd,
                D3DCREATE_SOFTWARE_VERTEXPROCESSING | D3DCREATE_DISABLE_DRIVER_MANAGEMENT,
                &mut present_parameters,
                &mut device,
            );
            let device_ref = device.as_ref().unwrap();
            println!("HRESULT -> {}", h_result);
            println!("Got d3d9 device -> {:?}", device);

            let end_scene_addr = IDirect3DDevice9::EndScene as *const ();
            println!("EndScene -> {:?}", end_scene_addr);

            device
        };

        // let vmt_addr = unsafe {
        //     pattern_scan(
        //         module,
        //         0x128000,
        //         &[
        //             Some(0xC7),
        //             Some(0x06),
        //             None,
        //             None,
        //             None,
        //             None,
        //             Some(0x89),
        //             Some(0x86),
        //             None,
        //             None,
        //             None,
        //             None,
        //             Some(0x89),
        //             Some(0x86),
        //         ],
        //     )
        //     .expect("Pattern scan for d3d9 vmt failed")
        //     .offset(2)
        // };

        // EndScene
        {
            let end_scene_addr = unsafe { std::ptr::null::<()>() as *mut c_void };
            // let end_scene_addr = unsafe { vmt_addr.offset(42) };
            // let end_scene = unsafe { std::mem::transmute::<_, EndSceneFn>(end_scene_addr) };
            // println!("End Scene address: {:p}", end_scene_addr);

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
                        D3D9HookInstallError::VirtualProtectVMT,
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
                        D3D9HookInstallError::VirtualProtectVMTRestore,
                    ));
                }
            }
        }

        self.installed = true;
        Ok(())
    }

    fn uninstall(&mut self) -> Result<(), UninstallError<D3D9HookUninstallError>> {
        if !self.installed {
            return Err(UninstallError::NotInstalled);
        }
        self.installed = false;
        Ok(())
    }
}

impl Drop for D3D9Hook {
    fn drop(&mut self) {
        if !self.installed {
            return;
        }
        self.uninstall().expect("Could not uninstall");
    }
}
