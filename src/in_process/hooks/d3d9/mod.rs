use crate::in_process::hooks::{Hook, Hookable, InstallError, UninstallError};
use detour::static_detour;
use log::debug;
use std::ffi::CString;
use std::ops::DerefMut;
use std::sync::Mutex;
use winapi::ctypes::c_void;
use winapi::shared::d3d9::*;
use winapi::shared::d3d9types::*;
use winapi::shared::minwindef::*;
use winapi::shared::ntdef::HRESULT;
use winapi::shared::windef::*;
use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::processthreadsapi::GetCurrentProcessId;
use winapi::um::winuser::{EnumWindows, GetWindowThreadProcessId};

const MODULE_NAME: &'static str = "d3d9.dll";

#[derive(Debug, thiserror::Error)]
pub enum D3D9HookInstallError {
    #[error("Module not found")]
    ModuleNotFound,
}

#[derive(Debug, thiserror::Error)]
pub enum D3D9HookUninstallError {
    #[error("Other")]
    Other,
}

type FnEndScene = unsafe extern "system" fn(device: *mut IDirect3DDevice9) -> HRESULT;

static_detour! {
    static EndSceneHook: unsafe extern "system" fn(*mut IDirect3DDevice9) -> HRESULT;
}

lazy_static::lazy_static! {
    static ref DEVICE_HOOK_CALLBACK: Mutex<Option<DeviceHookCallback>> = {
        Mutex::new(None)
    };
}

struct DeviceHookCallback(Box<dyn FnMut(&mut IDirect3DDevice9)>);
unsafe impl Send for DeviceHookCallback {}
unsafe impl Sync for DeviceHookCallback {}

unsafe extern "system" fn end_scene_detour(device: *mut IDirect3DDevice9) -> HRESULT {
    let device_ref = device.as_mut().expect("Could not get device as ref");

    {
        let mut device_hook_callback_guard = DEVICE_HOOK_CALLBACK
            .lock()
            .expect("Failed to lock current d3d9 hook");
        if let Some(device_hook_callback) = device_hook_callback_guard.deref_mut() {
            let callback = &mut device_hook_callback.0;
            callback(device_ref);
        }
    }

    let rect_color = D3DCOLOR_COLORVALUE(1.0, 0.0, 0.0, 1.0);
    let rect = D3DRECT {
        x1: 10,
        y1: 10,
        x2: 100,
        y2: 100,
    };

    device_ref.Clear(1, &rect, D3DCLEAR_TARGET, rect_color, 0.0, 0);
    EndSceneHook.call(device)
}

pub struct D3D9Hook {
    installed: bool,
    context: Option<*mut IDirect3D9>,
    device: Option<*mut IDirect3DDevice9>,
}

impl D3D9Hook {
    pub fn new() -> Self {
        Self {
            installed: false,
            context: None,
            device: None,
        }
    }

    pub fn set_device_hook_callback(callback: Box<dyn FnMut(&mut IDirect3DDevice9)>) {
        let device_hook_callback = DeviceHookCallback(callback);
        let mut dev_cb = DEVICE_HOOK_CALLBACK
            .lock()
            .expect("Failed to lock current device hook callback");
        *dev_cb = Some(device_hook_callback);
    }

    pub fn context(&self) -> Option<*mut IDirect3D9> {
        self.context
    }

    pub fn device(&self) -> Option<*mut IDirect3DDevice9> {
        self.device
    }
}

unsafe impl Send for D3D9Hook {}
unsafe impl Sync for D3D9Hook {}

impl Hookable for D3D9Hook {
    fn is_hookable() -> bool {
        let module_name =
            CString::new(MODULE_NAME).expect("Could not turn MODULE_NAME into a C string");
        let module = unsafe { GetModuleHandleA(module_name.as_ptr()) };

        module != std::ptr::null_mut()
    }
}

impl Hook<D3D9HookInstallError, D3D9HookUninstallError> for D3D9Hook {
    fn is_installed(&self) -> bool {
        self.installed
    }

    unsafe fn install(&mut self) -> Result<(), InstallError<D3D9HookInstallError>> {
        debug!("Attempting to install d3d9 hook");
        if self.installed {
            debug!("d3d9 hook already installed");
            return Err(InstallError::AlreadyInstalled);
        }

        let module_name =
            CString::new(MODULE_NAME).expect("Could not turn MODULE_NAME into a C string");
        let module = GetModuleHandleA(module_name.as_ptr());

        if module == std::ptr::null_mut() {
            debug!("Could not find d3d9.dll");
            return Err(InstallError::Other(D3D9HookInstallError::ModuleNotFound));
        }
        debug!("Module handle d3d9.dll addr: {:p}", module);
        let output = get_d3d_device().expect("Could not get device");
        debug!("Device: {:?}", output.device);
        let device_ptr = *std::mem::transmute::<_, *const *const c_void>(output.device);
        debug!("Device ptr: {:?}", device_ptr);
        let end_scene_addr = get_d3d_device_vmt_method_address(device_ptr, VmtMethod::EndScene);
        debug!("EndScene addr: {:?}", end_scene_addr);
        let end_scene: FnEndScene = std::mem::transmute(end_scene_addr);
        debug!("EndScene located at: {:?}", end_scene as *const ());

        EndSceneHook
            .initialize(end_scene, |device| end_scene_detour(device))
            .expect("Couldn't initialize EndScene hook");
        EndSceneHook
            .enable()
            .expect("Couldn't enable EndScene hook");

        self.context = Some(output.context);
        self.device = Some(output.device);

        self.installed = true;
        debug!("Installed d3d9 hook");
        Ok(())
    }

    unsafe fn uninstall(&mut self) -> Result<(), UninstallError<D3D9HookUninstallError>> {
        debug!("Attempting to uninstall d3d9 hook");
        if !self.installed {
            debug!("d3d9 was not installed");
            return Err(UninstallError::NotInstalled);
        }
        EndSceneHook
            .disable()
            .expect("Could not disable EndScene hook");
        self.device
            .as_ref()
            .expect("Device was not set")
            .as_mut()
            .expect("Could not get ref")
            .Release();
        self.context
            .as_ref()
            .expect("Context was not set")
            .as_mut()
            .expect("Could not get ref")
            .Release();

        self.installed = false;
        debug!("Uninstalled d3d9 hook");
        Ok(())
    }
}

// impl Drop for D3D9Hook {
//     fn drop(&mut self) {
//         if !self.installed {
//             return;
//         }
//         unsafe {
//             self.uninstall().expect("Could not uninstall");
//         }
//     }
// }

#[derive(Debug, thiserror::Error)]
enum GetD3DDeviceError {
    #[error("Could not create d3d")]
    D3DCreationFailed,
    #[error("Could not create device (code: {})", failure_code)]
    DeviceCreationFailed { failure_code: i32 },
}

struct GetProcessWindowWindowValueWrapper(*mut HWND__);

unsafe impl Send for GetProcessWindowWindowValueWrapper {}
unsafe impl Sync for GetProcessWindowWindowValueWrapper {}

unsafe extern "system" fn get_process_window_enum_windows_callback(
    handle: *mut HWND__,
    lparam: LPARAM,
) -> BOOL {
    let window_ptr: *mut Mutex<Option<GetProcessWindowWindowValueWrapper>> =
        std::mem::transmute(lparam);
    let window = window_ptr.as_mut().expect("Failed to get window from ptr");

    let mut wnd_proc_id: DWORD = std::mem::zeroed();
    GetWindowThreadProcessId(handle, &mut wnd_proc_id);
    if GetCurrentProcessId() != wnd_proc_id {
        // Skip to next window
        return 1;
    }

    {
        let mut window = window.lock().expect("Failed to lock window mutex");
        *window = Some(GetProcessWindowWindowValueWrapper(handle));
    }

    // Window was found
    0
}

unsafe fn get_process_window() -> Result<*mut HWND__, ()> {
    let mut window: Mutex<Option<GetProcessWindowWindowValueWrapper>> = Mutex::new(None);
    {
        let window_ptr = &mut window as *mut _;

        EnumWindows(
            Some(get_process_window_enum_windows_callback),
            std::mem::transmute(window_ptr),
        );
    }
    let window = window.lock().expect("Failed to lock window mutex");
    let value_wrapper = window.as_ref().ok_or_else(|| ())?;
    Ok(value_wrapper.0)
}

struct GetD3DDeviceOutput {
    context: *mut IDirect3D9,
    device: *mut IDirect3DDevice9,
}

impl Drop for GetD3DDeviceOutput {
    fn drop(&mut self) {
        unsafe {
            let device = self.device.as_ref().expect("Could not get device ptr");
            device.Release();
            let context = self.context.as_ref().expect("Could not get context ptr");
            context.Release();
        }
    }
}

unsafe fn get_d3d_device() -> Result<GetD3DDeviceOutput, GetD3DDeviceError> {
    let d3d = Direct3DCreate9(D3D_SDK_VERSION);
    if d3d == std::ptr::null_mut() {
        return Err(GetD3DDeviceError::D3DCreationFailed);
    }
    let d3d_ref = d3d.as_ref().expect("Invalid d3d pointer");

    let mut dummy_device: *mut IDirect3DDevice9 = std::ptr::null_mut();
    let mut d3dpp: D3DPRESENT_PARAMETERS = std::mem::zeroed();
    d3dpp.Windowed = 0;
    d3dpp.SwapEffect = D3DSWAPEFFECT_DISCARD;
    d3dpp.hDeviceWindow = get_process_window().expect("Failed to get process window");
    debug!(
        "Handle device window (get process window): {:?}",
        d3dpp.hDeviceWindow
    );

    let mut dummy_device_created = d3d_ref.CreateDevice(
        D3DADAPTER_DEFAULT,
        D3DDEVTYPE_HAL,
        d3dpp.hDeviceWindow,
        D3DCREATE_SOFTWARE_VERTEXPROCESSING,
        &mut d3dpp as *mut _,
        &mut dummy_device,
    );

    if dummy_device_created != 0 {
        d3dpp.Windowed = 1;

        dummy_device_created = d3d_ref.CreateDevice(
            D3DADAPTER_DEFAULT,
            D3DDEVTYPE_HAL,
            d3dpp.hDeviceWindow,
            D3DCREATE_SOFTWARE_VERTEXPROCESSING,
            &mut d3dpp as *mut _,
            &mut dummy_device,
        );
        if dummy_device_created != 0 {
            d3d_ref.Release();
            return Err(GetD3DDeviceError::DeviceCreationFailed {
                failure_code: dummy_device_created,
            });
        }
    }
    // let device_ptr = dummy_device;
    // dbg!(std::mem::transmute::<_, *const *const ()>(device_ptr));
    // let mut dummy_device = dummy_device.as_mut().expect("Dummy device invalid pointer");

    // dummy_device.Release();
    // d3d.Release();

    Ok(GetD3DDeviceOutput {
        context: d3d,
        device: dummy_device,
    })
}

unsafe fn get_d3d_device_vmt_method_address(
    device: *const c_void,
    method: VmtMethod,
) -> *mut c_void {
    let ptr = device as *mut u32;
    *(ptr.offset(isize::from(method as u8)) as *const *mut c_void)
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
