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
        elertan_cheat_base::log::debug!("Run started!");

        D3D9Hook::set_device_hook_callback(Box::new(|device| {}));
        let mut d3d9_hook = D3D9Hook::new();
        unsafe {
            d3d9_hook
                .install()
                .map_err(|_| AttachError::Custom(AppAttachError::Other))?
        };

        self.d3d9_hook = Some(d3d9_hook);

        Ok(())
    }

    fn detach(&mut self) -> Result<(), DetachError<AppDetachError>> {
        elertan_cheat_base::log::debug!("Cleaning up...");
        let d3d9_hook = self.d3d9_hook.as_mut().expect("D3D9 hook was not set");
        unsafe {
            d3d9_hook
                .uninstall()
                .map_err(|_| DetachError::Custom(AppDetachError::Other))?
        };

        Ok(())
    }
}

static APP: Lazy<Mutex<App>> = Lazy::new(|| Mutex::new(App::new()));

elertan_cheat_base::generate_entrypoint!(APP);
