use elertan_cheat_base::injection::helpers::AlertDialog;
use elertan_cheat_base::injection::hooks::d3d9::D3D9Hook;
use elertan_cheat_base::injection::hooks::Hook;

fn attach() {
    elertan_cheat_base::injection::console::open_console();

    let mut d3d9_hook = D3D9Hook::new();
    d3d9_hook.install().unwrap();
}

fn detach() {
    elertan_cheat_base::injection::console::close_console();
}

elertan_cheat_base::make_entrypoint!(attach, detach);
