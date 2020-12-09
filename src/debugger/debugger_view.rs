use std::sync::{Arc, Mutex};

use cursive::Printer;

use crate::debugger::debugger::Debugger;

pub struct DebuggerView {
    debugger: Arc<Mutex<Debugger>>,
}

impl DebuggerView {
    pub fn new(debugger: Arc<Mutex<Debugger>>) -> Self {
        DebuggerView { debugger: debugger }
    }
}

impl cursive::view::View for DebuggerView {
    fn draw(&self, printer: &Printer) {
        let d = self.debugger.lock().unwrap();

        printer.print((0, 0), "ASM:");

        for (i, (pc, instr)) in d.disassemble().iter().enumerate() {
            let mut data_str = String::new();
            if instr.length() == 1 {
                data_str = format!("{:#04X}", instr.byte());
            } else if instr.length() == 2 {
                data_str = format!("{:#06X}", instr.word());
            }

            let mut opcode_str = String::new();

            match instr.opcode() {
                Some(opcode) => opcode_str = format!("{:?}", opcode),
                None => opcode_str = "Unknonwn".to_string(),
            }

            printer.print(
                (0, i + 1),
                format!("{:#06X}: {} {}", pc, opcode_str, data_str).as_str(),
            );
        }
    }
}
