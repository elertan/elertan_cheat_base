use once_cell::sync::OnceCell;
use std::sync::{mpsc, Mutex};
use std::sync::mpsc::{Sender};

static RUN_THREAD_KILL_SENDER: OnceCell<Mutex<Sender<()>>> = OnceCell::new();

fn attach() {
    let (tx, rx) = mpsc::channel();
    RUN_THREAD_KILL_SENDER.set(Mutex::new(tx)).expect("Failed to set thread kill sender");
    std::thread::spawn(move || {
        println!("Running!");

        // Once this will receive a message the thread will get killed (blocks until)
        rx.recv().expect("Failed to receive run thread kill command");
    });
}

fn detach() {
    let tx = RUN_THREAD_KILL_SENDER.get().expect("Failed to get run thread kill sender");
    let tx = tx.lock().expect("Failed to acquire run thread kill sender");
    tx.send(()).expect("Failed to send kill command to run thread");
}

elertan_cheat_base::make_entrypoint!(attach, detach);

