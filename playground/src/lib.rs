use elertan_cheat_base::injection::helpers::AlertDialog;

fn attach() {
    let dialog = AlertDialog::new()
        .title("Simple Injection")
        .message("It worked!");
    dialog.show().expect("Failed to show dialog");
}

fn detach() {
}

elertan_cheat_base::make_entrypoint!(attach, detach);

