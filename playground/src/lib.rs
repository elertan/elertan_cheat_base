use elertan_cheat_base::injection::hooks::d3d9::D3D9Hook;
use elertan_cheat_base::injection::hooks::Hook;
use once_cell::sync::OnceCell;

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
        .level(elertan_cheat_base::log::LevelFilter::Debug)
        // .chain(std::io::stdout())
        .chain(fern::log_file("elertan_cheat_base_playground.log")?)
        .apply()?;
    Ok(())
}

static mut D3D9_HOOK: OnceCell<D3D9Hook> = OnceCell::new();

fn attach() {
    setup_logger().expect("Failed to set up logger");
    // elertan_cheat_base::injection::console::open_console();

    let mut d3d9_hook = D3D9Hook::new();
    unsafe {
        d3d9_hook.install().unwrap();
        let _ = D3D9_HOOK.set(d3d9_hook);
    }
}

fn detach() {
    unsafe {
        let d3d9_hook = D3D9_HOOK.get_mut().unwrap();
        d3d9_hook.uninstall().unwrap();
    }
    // elertan_cheat_base::injection::console::close_console();
}

elertan_cheat_base::make_entrypoint!(attach, detach);
