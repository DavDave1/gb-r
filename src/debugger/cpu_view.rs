use crate::gbr::cpu::CpuState;

pub fn show(state: &CpuState, ui: &mut egui::Ui) {
    ui.heading("CPU");
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
