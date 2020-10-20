use elertan_cheat_base::injection::{find_process_id_by_process_name, inject_dll_into_process};
use std::path::Path;
use std::time::Duration;

fn main() {
    let pid = find_process_id_by_process_name("notepad.exe").unwrap_or_else(|| {
        println!("Notepad was not found");
        std::thread::sleep(Duration::from_secs(10));
        panic!();
    });

    println!("Notepad process found: {}", pid);
    let dll_path = Path::new("../../simple_injection.dll");
    println!(
        "Dll path: {}",
        dll_path.canonicalize().unwrap().to_str().unwrap()
    );

    let result = unsafe { inject_dll_into_process(pid, dll_path) };
    dbg!(result);
    std::thread::sleep(Duration::from_secs(10));
}
