use std::sync::{Arc, Mutex};

use cursive::Printer;

use crate::debugger::debugger::Debugger;

pub struct AsmView {
    debugger: Arc<Mutex<Debugger>>,
}

impl AsmView {
    pub fn new(debugger: Arc<Mutex<Debugger>>) -> Self {
        AsmView { debugger: debugger }
    }
}

impl cursive::view::View for AsmView {
    fn draw(&self, printer: &Printer) {
        let d = self.debugger.lock().unwrap();

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
                (1, i),
                format!("{:#06X}: {} {}", pc, opcode_str, data_str).as_str(),
            );
        }
    }
}
