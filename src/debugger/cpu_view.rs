use std::sync::Arc;

use crate::debugger::debugger::Debugger;

pub fn show(debugger: Arc<Debugger>, ui: &mut egui::Ui) {

        let state = debugger.cpu_state();

        ui.label("Registers:");

        let label = format!(
                "AF: {:#06X}, BC: {:#06X}, DE: {:#06X}, HL: {:#06X}, PC: {:#06X}, SP: {:#06X}",
                state.af, state.bc, state.de, state.hl, state.pc, state.sp,
            );

    ui.label(label);

    ui.label("Flags:");
    ui.label(format!(
                "Z: {}, C: {}, BCD-N: {}, BCD-H: {}",
                state.zero, state.carry, state.bcd_n, state.bcd_h
            ));

}

