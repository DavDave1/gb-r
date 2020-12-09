use crate::debugger::debugger::Debugger;

pub enum Command {
    Step,
    Quit,
}

pub fn command_step(siv: &mut cursive::Cursive, debugger: &mut Debugger) {
    debugger.step();
}
