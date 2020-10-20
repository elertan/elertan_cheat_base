use elertan_cheat_base::injection::inject_dll_into_process;
use std::path::Path;
use std::time::Duration;
use sysinfo::{ProcessExt, SystemExt};

fn main() {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();

    let processes = system.get_processes();
    let notepad_result = processes
        .into_iter()
        .find(|(_pid, proc)| proc.name() == "notepad.exe")
        .unwrap_or_else(|| {
            println!("Notepad was not found");
            std::thread::sleep(Duration::from_secs(10));
            panic!();
        });

    let process_id = *notepad_result.0 as u32;
    println!("Notepad process found: {}", process_id);
    let dll_path = Path::new("../../simple_injection.dll");
    println!(
        "Dll path: {}",
        dll_path.canonicalize().unwrap().to_str().unwrap()
    );

    let result = unsafe { inject_dll_into_process(process_id, dll_path) };
    dbg!(result);
    std::thread::sleep(Duration::from_secs(10));
}
