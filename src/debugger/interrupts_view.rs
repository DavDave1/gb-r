use crate::gbr::interrupts::InterruptHandlerState;

pub fn show(ir_hander: &mut InterruptHandlerState, ui: &mut egui::Ui) {
    ui.heading("Interrupts");
    ui.horizontal(|ui| {
        ui.label("Vblank: ");
        ui.checkbox(&mut ir_hander.vblank.enabled, "Enabled");
        ui.checkbox(&mut ir_hander.vblank.set, "Set");
    });

    ui.horizontal(|ui| {
        ui.label("LCD stat: ");
        ui.checkbox(&mut ir_hander.lcd_stat.enabled, "Enabled");
        ui.checkbox(&mut ir_hander.lcd_stat.set, "Set");
    });

    ui.horizontal(|ui| {
        ui.label("Timer: ");
        ui.checkbox(&mut ir_hander.timer.enabled, "Enabled");
        ui.checkbox(&mut ir_hander.timer.set, "Set");
    });

    ui.horizontal(|ui| {
        ui.label("Serial: ");
        ui.checkbox(&mut ir_hander.serial.enabled, "Enabled");
        ui.checkbox(&mut ir_hander.serial.set, "Set");
    });

    ui.horizontal(|ui| {
        ui.label("Joypad: ");
        ui.checkbox(&mut ir_hander.joypad.enabled, "Enabled");
        ui.checkbox(&mut ir_hander.joypad.set, "Set");
    });
}
