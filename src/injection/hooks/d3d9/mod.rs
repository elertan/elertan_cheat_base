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
type FnEndScene = unsafe extern "system" fn() -> HRESULT;

detour::static_detour! {
    static EndSceneHook: unsafe extern "system" fn() -> HRESULT;
}

extern "system" fn end_scene_detour() -> HRESULT {
    unsafe { EndSceneHook.call() }
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
        let module = unsafe { GetModuleHandleA(module_name.as_ptr()) };
        if module == std::ptr::null_mut() {
            return Err(InstallError::Custom(D3D9HookInstallError::ModuleNotFound));
        }
        println!("Module handle d3d9.dll addr: {:p}", module);

        let vmt_pattern_scan_addr = unsafe {
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
            .expect("Pattern scan for d3d9 vmt failed")
        };
        dbg!(vmt_pattern_scan_addr);
        let vmt_pattern_scan_vmt_addr_offset = unsafe { vmt_pattern_scan_addr.offset(2) };
        dbg!(vmt_pattern_scan_vmt_addr_offset);
        let vmt_addr = unsafe { *(vmt_pattern_scan_vmt_addr_offset as *const u32) as *const u8 };
        dbg!(vmt_addr);

        let end_scene_idx = VmtMethods::EndScene as u8;
        dbg!(end_scene_idx);
        let end_scene_addr = unsafe { vmt_addr.offset(isize::from(end_scene_idx)) as *const u32 };
        dbg!(end_scene_addr);
        let end_scene: FnEndScene = unsafe { std::mem::transmute(end_scene_addr) };

        let mut end_scene_hook = EndSceneHook
            .initialize(end_scene, end_scene_detour)
            .expect("Couldn't initialize EndScene hook");
        end_scene_hook
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

#[repr(u8)]
#[allow(unused)]
enum VmtMethods {
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
