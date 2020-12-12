use crate::debugger::debugger::Debugger;

pub enum Command {
    RunStop,
    RunDetached,
    Step,
    Quit,
}

pub fn command_run_stop(siv: &mut cursive::Cursive, debugger: &mut Debugger) {
    debugger.set_running(!debugger.is_running());
}

pub fn command_run_detached(siv: &mut cursive::Cursive, debugger: &mut Debugger) {
    debugger.set_running(true);
    while debugger.is_running() {
        debugger.step();
    }
}

pub fn command_step(siv: &mut cursive::Cursive, debugger: &mut Debugger) {
    debugger.step();
}
