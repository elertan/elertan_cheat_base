use simple_logger::SimpleLogger;
use std::path::Path;

fn main() {
    SimpleLogger::new().init().unwrap();

    let process_name = "ac_client.exe";

    let pid = 0u32;
    let dll_path = Path::new("../../injected/target/");
    log::info!("Injecting simple_godmode into ac_client.exe");
}
