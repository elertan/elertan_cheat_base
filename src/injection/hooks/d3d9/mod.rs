use crate::injection::hooks::{Hook, InstallError, UninstallError};
use crate::injection::memory::pattern_scan;
use detour::static_detour;
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
    // VirtualProtectVMT,
    // VirtualProtectVMTRestore,
}

#[derive(Debug)]
pub enum D3D9HookUninstallError {
    Other,
}

// type Direct3D9Create9Fn = unsafe extern "system" fn(version: UINT) -> *mut IDirect3D9;
type FnEndScene = unsafe extern "system" fn(device: *mut IDirect3DDevice9) -> HRESULT;

static_detour! {
    static EndSceneHook: unsafe extern "system" fn(*mut IDirect3DDevice9) -> HRESULT;
}

extern "system" fn end_scene_detour(device: *mut IDirect3DDevice9) -> HRESULT {
    println!("EndScene was called!");
    unsafe { EndSceneHook.call(device) }
}

// #[no_mangle]
// unsafe extern "system" fn tmp_wnd_proc(
//     hwnd: HWND,
//     uint: UINT,
//     wparam: WPARAM,
//     lparam: LPARAM,
// ) -> LRESULT {
//     DefWindowProcA(hwnd, uint, wparam, lparam)
// }

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

    unsafe fn install(&mut self) -> Result<(), InstallError<D3D9HookInstallError>> {
        if self.installed {
            return Err(InstallError::AlreadyInstalled);
        }

        let module_name =
            CString::new(MODULE_NAME).expect("Could not turn MODULE_NAME into a C string");
        let module = GetModuleHandleA(module_name.as_ptr());

        if module == std::ptr::null_mut() {
            return Err(InstallError::Custom(D3D9HookInstallError::ModuleNotFound));
        }
        println!("Module handle d3d9.dll addr: {:p}", module);

        // let vmt_pattern_scan_addr = unsafe {
        //     pattern_scan(
        //         module as *const u8,
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
        // };
        // dbg!(vmt_pattern_scan_addr);
        // let vmt_pattern_scan_vmt_addr_offset = unsafe { vmt_pattern_scan_addr.offset(2) };
        // dbg!(vmt_pattern_scan_vmt_addr_offset);
        // let vmt_addr = unsafe { *(vmt_pattern_scan_vmt_addr_offset as *const u32) as *const u8 };
        // dbg!(vmt_addr);
        //
        // let end_scene_idx = VmtMethod::EndScene as u8;
        // dbg!(end_scene_idx);
        // let end_scene_addr = unsafe { vmt_addr.offset(isize::from(end_scene_idx)) as *const u32 };
        let device = get_d3d_device().expect("Could not get device");
        let end_scene_addr = get_d3d_device_vmt_method_address(device, VmtMethod::EndScene);
        dbg!(end_scene_addr);
        let end_scene: FnEndScene = std::mem::transmute(end_scene_addr);

        EndSceneHook
            .initialize(end_scene, |device| end_scene_detour(device))
            .expect("Couldn't initialize EndScene hook");
        EndSceneHook
            .enable()
            .expect("Couldn't enable EndScene hook");
        // let vmt = unsafe { std::mem::transmute::<_, &[u32; 120]>(*vmt_pattern_scan_addr) };
        // let end_scene_idx = VmtMethods::EndScene as usize;
        // dbg!(end_scene_idx);
        //
        // // EndScene
        // {
        //     let end_scene_addr = unsafe { vmt[end_scene_idx] };
        //     let end_scene = unsafe { std::mem::transmute::<_, EndSceneFn>(end_scene_addr) };
        //     println!("End Scene address: {:?}", end_scene_addr as *mut ());
        //
        //     let mut old_protection: DWORD = 0;
        //     // Get access to write vmt memory area
        //     {
        //         let success = unsafe {
        //             VirtualProtect(
        //                 end_scene_addr as *mut c_void,
        //                 4,
        //                 PAGE_EXECUTE_READWRITE,
        //                 &mut old_protection,
        //             )
        //         };
        //         if success == 0 {
        //             return Err(InstallError::Custom(
        //                 D3D9HookInstallError::VirtualProtectVMT,
        //             ));
        //         }
        //     }
        //
        //     unsafe {
        //         *(end_scene_addr as *mut u32) = std::mem::transmute::<_, u32>(&end_scene_hook);
        //     }
        //
        //     // Restore previous protection for vmt memory area
        //     {
        //         let success = unsafe {
        //             VirtualProtect(
        //                 end_scene_addr as *mut c_void,
        //                 4,
        //                 old_protection,
        //                 &mut old_protection,
        //             )
        //         };
        //         if success == 0 {
        //             return Err(InstallError::Custom(
        //                 D3D9HookInstallError::VirtualProtectVMTRestore,
        //             ));
        //         }
        //     }
        // }

        self.installed = true;
        Ok(())
    }

    unsafe fn uninstall(&mut self) -> Result<(), UninstallError<D3D9HookUninstallError>> {
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
        unsafe {
            self.uninstall().expect("Could not uninstall");
        }
    }
}

#[derive(Debug, snafu::Snafu)]
enum GetD3DDeviceError {
    #[snafu(display("Could not create d3d"))]
    D3DCreationFailed,
    #[snafu(display("Could not create device"))]
    DeviceCreationFailed,
}

unsafe fn get_process_window() -> *mut HWND__ {
    unimplemented!();
}

unsafe fn get_d3d_device() -> Result<*mut IDirect3DDevice9, GetD3DDeviceError> {
    let d3d = Direct3DCreate9(D3D_SDK_VERSION);
    if d3d == std::ptr::null_mut() {
        return Err(GetD3DDeviceError::D3DCreationFailed);
    }
    let d3d = d3d.as_ref().expect("Invalid d3d pointer");

    let mut dummy_device: *mut IDirect3DDevice9 = std::ptr::null_mut();
    let mut d3dpp: D3DPRESENT_PARAMETERS = std::mem::zeroed();
    d3dpp.Windowed = 0;
    d3dpp.SwapEffect = D3DSWAPEFFECT_DISCARD;
    d3dpp.hDeviceWindow = get_process_window();

    let mut dummy_device_created = d3d.CreateDevice(
        D3DADAPTER_DEFAULT,
        D3DDEVTYPE_HAL,
        d3dpp.hDeviceWindow,
        D3DCREATE_SOFTWARE_VERTEXPROCESSING,
        &mut d3dpp as *mut _,
        &mut dummy_device,
    );

    if dummy_device_created != 0 {
        d3dpp.Windowed = 1;

        dummy_device_created = d3d.CreateDevice(
            D3DADAPTER_DEFAULT,
            D3DDEVTYPE_HAL,
            d3dpp.hDeviceWindow,
            D3DCREATE_SOFTWARE_VERTEXPROCESSING,
            &mut d3dpp as *mut _,
            &mut dummy_device,
        );
        if dummy_device_created != 0 {
            return Err(GetD3DDeviceError::DeviceCreationFailed);
        }
    }
    let dummy_device = dummy_device.as_ref().expect("Dummy device invalid pointer");

    dummy_device.Release();
    d3d.Release();

    Ok(std::ptr::null_mut())
}

unsafe fn get_d3d_device_vmt_method_address(
    device: *mut IDirect3DDevice9,
    method: VmtMethod,
) -> *mut c_void {
    todo!()
}

#[repr(u8)]
#[allow(unused)]
enum VmtMethod {
    QueryInterface,
    AddRef,
    Release,
    TestCooperativeLevel,
    GetAvailableTextureMem,
    EvictManagedResources,
    GetDirect3D,
    GetDeviceCaps,
    GetDisplayMode,
    GetCreationParameters,
    SetCursorProperties,
    SetCursorPosition,
    ShowCursor,
    CreateAdditionalSwapChain,
    GetSwapChain,
    GetNumberOfSwapChains,
    Reset,
    Present,
    GetBackBuffer,
    GetRasterStatus,
    SetDialogBoxMode,
    SetGammaRamp,
    GetGammaRamp,
    CreateTexture,
    CreateVolumeTexture,
    CreateCubeTexture,
    CreateVertexBuffer,
    CreateIndexBuffer,
    CreateRenderTarget,
    CreateDepthStencilSurface,
    UpdateSurface,
    UpdateTexture,
    GetRenderTargetData,
    GetFrontBufferData,
    StretchRect,
    ColorFill,
    CreateOffscreenPlainSurface,
    SetRenderTarget,
    GetRenderTarget,
    SetDepthStencilSurface,
    GetDepthStencilSurface,
    BeginScene,
    EndScene,
    Clear,
    SetTransform,
    GetTransform,
    MultiplyTransform,
    SetViewport,
    GetViewport,
    SetMaterial,
    GetMaterial,
    SetLight,
    GetLight,
    LightEnable,
    GetLightEnable,
    SetClipPlane,
    GetClipPlane,
    SetRenderState,
    GetRenderState,
    CreateStateBlock,
    BeginStateBlock,
    EndStateBlock,
    SetClipStatus,
    GetClipStatus,
    GetTexture,
    SetTexture,
    GetTextureStageState,
    SetTextureStageState,
    GetSamplerState,
    SetSamplerState,
    ValidateDevice,
    SetPaletteEntries,
    GetPaletteEntries,
    SetCurrentTexturePalette,
    GetCurrentTexturePalette,
    SetScissorRect,
    GetScissorRect,
    SetSoftwareVertexProcessing,
    GetSoftwareVertexProcessing,
    SetNPatchMode,
    GetNPatchMode,
    DrawPrimitive,
    DrawIndexedPrimitive,
    DrawPrimitiveUP,
    DrawIndexedPrimitiveUP,
    ProcessVertices,
    CreateVertexDeclaration,
    SetVertexDeclaration,
    GetVertexDeclaration,
    SetFVF,
    GetFVF,
    CreateVertexShader,
    SetVertexShader,
    GetVertexShader,
    SetVertexShaderConstantF,
    GetVertexShaderConstantF,
    SetVertexShaderConstantI,
    GetVertexShaderConstantI,
    SetVertexShaderConstantB,
    GetVertexShaderConstantB,
    SetStreamSource,
    GetStreamSource,
    SetStreamSourceFreq,
    GetStreamSourceFreq,
    SetIndices,
    GetIndices,
    CreatePixelShader,
    SetPixelShader,
    GetPixelShader,
    SetPixelShaderConstantF,
    GetPixelShaderConstantF,
    SetPixelShaderConstantI,
    GetPixelShaderConstantI,
    SetPixelShaderConstantB,
    GetPixelShaderConstantB,
    DrawRectPatch,
    DrawTriPatch,
    DeletePatch,
    CreateQuery,
}
