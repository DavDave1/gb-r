use crate::gbr::io_registers::IORegisters;

pub fn show(io_regs: &IORegisters, ui: &mut egui::Ui) {
    ui.heading("IO Registers");
    ui.label(format!("Port P1: {:#010b}", io_regs.port_p1()));
}
