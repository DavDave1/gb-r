use crate::gbr::joypad::Joypad;

pub fn show(joypad: &Joypad, ui: &mut egui::Ui) {
    ui.heading("Joypad");
    ui.label(format!("Raw: {:#010b}", joypad.read()));
}
