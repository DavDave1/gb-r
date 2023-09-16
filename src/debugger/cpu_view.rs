use crate::gbr::cpu::CpuState;

pub fn show(state: &mut CpuState, ui: &mut egui::Ui) {
    ui.heading("CPU");
    ui.label("Registers:");

    let label = format!(
        "AF: {:#06X}, BC: {:#06X}, DE: {:#06X}, HL: {:#06X}, PC: {:#06X}, SP: {:#06X}",
        state.af, state.bc, state.de, state.hl, state.pc, state.sp,
    );

    ui.label(label);

    ui.horizontal_wrapped(|ui| {
        ui.label("Flags:");
        ui.checkbox(&mut state.zero, "Z");
        ui.checkbox(&mut state.carry, "C");
        ui.checkbox(&mut state.bcd_n, "N");
        ui.checkbox(&mut state.bcd_h, "H");
    });
}
