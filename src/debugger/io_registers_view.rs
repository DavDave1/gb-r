use crate::gbr::io_registers::IORegisters;

pub fn show(io_regs: &IORegisters, ui: &mut egui::Ui) {
    ui.heading("IO Registers");
    ui.label(format!("Port P1: {:#010b}", io_regs.port_p1()));

    ui.label(format!("Serial data: {:#010b}", io_regs.serial_data()));

    ui.label(format!(
        "Serial control: {:#010b}",
        io_regs.serial_control()
    ));

    ui.label(format!("Sound enable: {:#010b}", io_regs.sound_enable()));

    ui.label(format!(
        "Sound ch1 wave pattern len: {:#010b}",
        io_regs.sound_ch1_wave_pattern_length()
    ));

    ui.label(format!(
        "Sound ch1 volume envelope: {:#010b}",
        io_regs.sound_ch1_volume_envelope()
    ));

    ui.label(format!(
        "Sound output terminal selection: {:#010b}",
        io_regs.sound_output_terminal_selection()
    ));

    ui.label(format!(
        "Sound channel volume control: {:#010b}",
        io_regs.sound_channel_volume_control()
    ));
}
