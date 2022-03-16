use std::sync::mpsc::Sender;

pub enum Command {
    RunStop,
    Step,
    Quit,
}

pub fn command_run_stop(siv: &mut cursive::Cursive, start_sig: Sender<i64>) {
    start_sig.send(-1).unwrap();
    log::info!("debugger is running");
}

pub fn command_step(siv: &mut cursive::Cursive, start_sig: Sender<i64>) {
    start_sig.send(1).unwrap();
}
