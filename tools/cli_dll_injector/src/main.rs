use elertan_cheat_base::injection::{find_process_id_by_process_name, inject_dll_into_process};
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(about = "Injects a dll into the target process")]
struct Opt {
    #[structopt(long, required_unless("process-name"), help = "The process id")]
    pub pid: Option<u32>,

    #[structopt(
        short,
        long,
        required_unless("pid"),
        help = "The name of the process (e.g. \"notepad.exe\")"
    )]
    pub process_name: Option<String>,

    #[structopt(
        name = "DLL_FILE_PATH",
        parse(from_os_str),
        help = "The path to the dll that will be injected"
    )]
    pub dll_path: PathBuf,
}

fn main() {
    let opt: Opt = Opt::from_args();

    let process_id = if let Some(pid) = opt.pid {
        pid
    } else if let Some(process_name) = opt.process_name {
        find_process_id_by_process_name(process_name.as_str()).unwrap_or_else(|| {
            println!("Process \"{}\" was not found", process_name);
            panic!();
        })
    } else {
        panic!();
    };

    let path = opt.dll_path.as_path();
    let result = unsafe { inject_dll_into_process(process_id, path) };
    if let Err(err) = result {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}
