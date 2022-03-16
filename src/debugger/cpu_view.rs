use std::sync::Arc;

use cursive::Printer;

use crate::debugger::debugger::Debugger;

pub struct CpuView {
    debugger: Arc<Debugger>,
}

impl CpuView {
    pub fn new(debugger: Arc<Debugger>) -> Self {
        CpuView { debugger }
    }
}

impl cursive::view::View for CpuView {
    fn draw(&self, printer: &Printer) {
        let state = self.debugger.cpu_state();

        printer.print((0, 0), "Registers:");
        printer.print(
            (0, 1),
            format!(
                "AF: {:#06X}, BC: {:#06X}, DE: {:#06X}, HL: {:#06X}, PC: {:#06X}, SP: {:#06X}",
                state.af, state.bc, state.de, state.hl, state.pc, state.sp,
            )
            .as_str(),
        );
        printer.print((0, 2), "");
        printer.print((0, 3), "Flags:");
        printer.print(
            (0, 4),
            format!(
                "Z: {}, C: {}, BCD-N: {}, BCD-H: {}",
                state.zero, state.carry, state.bcd_n, state.bcd_h
            )
            .as_str(),
        );
    }
}
