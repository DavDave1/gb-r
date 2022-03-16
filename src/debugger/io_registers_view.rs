use std::sync::Arc;

use cursive::Printer;

use crate::debugger::debugger::Debugger;

pub struct IORegistersView {
    debugger: Arc<Debugger>,
}

impl IORegistersView {
    pub fn new(debugger: Arc<Debugger>) -> Self {
        IORegistersView { debugger }
    }
}

impl cursive::view::View for IORegistersView {
    fn draw(&self, printer: &Printer) {
        let io_regs = self.debugger.io_registers();

        printer.print(
            (0, 0),
            format!("Port P1: {:#010b}", io_regs.port_p1()).as_str(),
        );
        printer.print(
            (0, 1),
            format!("Serial data: {:#010b}", io_regs.serial_data()).as_str(),
        );

        printer.print(
            (0, 2),
            format!("Serial control: {:#010b}", io_regs.serial_control()).as_str(),
        );

        printer.print(
            (0, 3),
            format!("Sound enable: {:#010b}", io_regs.sound_enable()).as_str(),
        );

        printer.print(
            (0, 4),
            format!(
                "Sound ch1 wave pattern len: {:#010b}",
                io_regs.sound_ch1_wave_pattern_length()
            )
            .as_str(),
        );

        printer.print(
            (0, 5),
            format!(
                "Sound ch1 volume envelope: {:#010b}",
                io_regs.sound_ch1_volume_envelope()
            )
            .as_str(),
        );

        printer.print(
            (0, 6),
            format!(
                "Sound output terminal selection: {:#010b}",
                io_regs.sound_output_terminal_selection()
            )
            .as_str(),
        );

        printer.print(
            (0, 7),
            format!(
                "Sound channel volume control: {:#010b}",
                io_regs.sound_channel_volume_control()
            )
            .as_str(),
        );

        printer.print(
            (0, 8),
            format!(
                "Background palette: {} - {} - {} - {}",
                io_regs.bg_palette().color_0().as_ascii(),
                io_regs.bg_palette().color_1().as_ascii(),
                io_regs.bg_palette().color_2().as_ascii(),
                io_regs.bg_palette().color_3().as_ascii()
            )
            .as_str(),
        );

        printer.print(
            (0, 9),
            format!("LCD control: {:#010b}", io_regs.lcd_control().raw()).as_str(),
        );
    }
}
