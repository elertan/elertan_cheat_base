use elertan_cheat_base::injection::helpers::AlertDialog;
use elertan_cheat_base::injection::hooks::direct_d3d9::DirectD3D9;
use elertan_cheat_base::injection::hooks::Hook;

fn attach() {
    let mut direct_d3d9_hook = DirectD3D9::new();
    direct_d3d9_hook.install().unwrap();
}

fn detach() {
}

elertan_cheat_base::make_entrypoint!(attach, detach);

