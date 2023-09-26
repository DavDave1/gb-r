use crate::gbr::mbc::MbcState;

pub fn show(state: &mut MbcState, ui: &mut egui::Ui) {
    ui.heading("MBC");
    ui.label(format!("Mapper type: {:?}", state.mbc_type));
    ui.label(format!(
        "Active ROM bank: {}/{}",
        state.active_rom_bank, state.rom_banks_count
    ));
    ui.label(format!(
        "Active RAM bank: {}/{}",
        state.active_ram_bank, state.ram_banks_count
    ));
    ui.checkbox(&mut state.ram_enable, "RAM enable");
    ui.label(format!("Banking mode: {:?}", state.banking_mode));
}
