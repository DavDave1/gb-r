use std::sync::Arc;

use crate::debugger::debugger::Debugger;

const COL_MIN_WIDTH: f32 = 20.0;
const COL_MAX_WIDTH: f32 = 200.0;

pub fn show(debugger: Arc<Debugger>, ui: &mut egui::Ui) {
    if let Some(asm) = debugger.asm_state() {
        egui::Grid::new("Asm View")
            .min_col_width(COL_MIN_WIDTH)
            .max_col_width(COL_MAX_WIDTH)
            .striped(true)
            .show(ui, |ui| {
                for (pc, instruction) in asm.iter() {
                    let label = match instruction {
                        Some(instr) => {
                            let opcode_str = match instr.opcode() {
                                Some(opcode) => format!("{:?}", opcode).to_string(),
                                None => "Unknonwn".to_string(),
                            };

                            let instr_len = instr.length().unwrap_or(0);

                            let data_str = if instr_len == 1 {
                                format!("{:#04X}", instr.byte())
                            } else {
                                format!("{:#06X}", instr.word())
                            };

                            format!("{:#06X}: {} {}", pc, opcode_str, data_str)
                        }
                        None => format!("{:#06X}: Unknonwn instruction", pc),
                    };
                    ui.label(label);
                    ui.end_row();
                }
            });
    }
}
