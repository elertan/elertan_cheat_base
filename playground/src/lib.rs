use elertan_cheat_base::imgui::{FontConfig, FontGlyphRanges, FontSource};
use elertan_cheat_base::injection::entrypoint::{AttachError, DetachError, Entrypoint};
use elertan_cheat_base::injection::hooks::d3d9::D3D9Hook;
use elertan_cheat_base::injection::hooks::Hook;
use elertan_cheat_base::injection::hooks::Hookable;
use once_cell::sync::{Lazy, OnceCell};
use std::sync::Mutex;

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%Y-%m-%d][%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(elertan_cheat_base::log::LevelFilter::Trace)
        // .chain(std::io::stdout())
        .chain(fern::log_file("elertan_cheat_base_playground.log")?)
        .apply()?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum AppAttachError {
    #[error("Other")]
    Other,
}

#[derive(Debug, thiserror::Error)]
pub enum AppDetachError {
    #[error("Other")]
    Other,
}

struct App {
    d3d9_hook: Option<D3D9Hook>,
}

impl App {
    pub fn new() -> Self {
        Self { d3d9_hook: None }
    }
}

impl Entrypoint<AppAttachError, AppDetachError> for App {
    fn attach(&mut self) -> Result<(), AttachError<AppAttachError>> {
        setup_logger().map_err(|_| AttachError::Custom(AppAttachError::Other))?;
        elertan_cheat_base::log::debug!("Attaching..");

        let mut d3d9_hook = D3D9Hook::new();
        unsafe {
            d3d9_hook
                .install()
                .map_err(|_| AttachError::Custom(AppAttachError::Other))?
        };

        self.d3d9_hook = Some(d3d9_hook);

        let mut imgui = elertan_cheat_base::imgui::Context::create();
        imgui.set_ini_filename(None);
        let mut platform = elertan_cheat_base::imgui_winit_support::WinitPlatform::init(&mut imgui);
        let hidpi_factor = platform.hidpi_factor();
        let font_size = (13.0 * hidpi_factor) as f32;
        imgui.fonts().add_font(&[
            FontSource::DefaultFontData {
                config: Some(FontConfig {
                    size_pixels: font_size,
                    ..FontConfig::default()
                }),
            },
            FontSource::TtfData {
                data: include_bytes!("../../resources/mplus-1p-regular.ttf"),
                size_pixels: font_size,
                config: Some(FontConfig {
                    rasterizer_multiply: 1.75,
                    glyph_ranges: FontGlyphRanges::japanese(),
                    ..FontConfig::default()
                }),
            },
        ]);
        imgui.fonts().build_rgba32_texture();

        imgui.io_mut().font_global_scale = (1.0 / hidpi_factor) as f32;
        imgui.io_mut().display_size = [400f32, 400f32];

        let device_ptr = self.d3d9_hook.as_ref().unwrap().device().unwrap();
        elertan_cheat_base::log::debug!("{:?}", &device_ptr);
        let com_ptr = unsafe { elertan_cheat_base::wio::com::ComPtr::from_raw(device_ptr) };
        elertan_cheat_base::log::debug!("{:?}", &com_ptr);

        let mut renderer = unsafe {
            elertan_cheat_base::imgui_dx9_renderer::Renderer::new(&mut imgui, com_ptr).unwrap()
        };

        D3D9Hook::set_device_hook_callback(Box::new(move |device| unsafe {
            let mut ui = imgui.frame();

            elertan_cheat_base::imgui::Window::new(elertan_cheat_base::imgui::im_str!(
                "Hello world"
            ))
            .size(
                [300.0, 100.0],
                elertan_cheat_base::imgui::Condition::FirstUseEver,
            )
            .build(&ui, || {
                ui.text(elertan_cheat_base::imgui::im_str!("Hello world!"));
                ui.text(elertan_cheat_base::imgui::im_str!("こんにちは世界！"));
                ui.text(elertan_cheat_base::imgui::im_str!("This...is...imgui-rs!"));
                ui.separator();
                let mouse_pos = ui.io().mouse_pos;
                ui.text(format!(
                    "Mouse Position: ({:.1},{:.1})",
                    mouse_pos[0], mouse_pos[1]
                ));
            });
            dbg!(ui.window_size());
            let draw_data = ui.render();
            // renderer.render(draw_data);
        }));
        elertan_cheat_base::log::debug!("Attached!");
        Ok(())
    }

    fn detach(&mut self) -> Result<(), DetachError<AppDetachError>> {
        elertan_cheat_base::log::debug!("Detaching...");
        let d3d9_hook = self.d3d9_hook.as_mut().expect("D3D9 hook was not set");
        unsafe {
            d3d9_hook
                .uninstall()
                .map_err(|_| DetachError::Custom(AppDetachError::Other))?
        };

        elertan_cheat_base::log::debug!("Detached!");
        Ok(())
    }
}

static APP: Lazy<Mutex<App>> = Lazy::new(|| Mutex::new(App::new()));

elertan_cheat_base::generate_entrypoint!(APP);
