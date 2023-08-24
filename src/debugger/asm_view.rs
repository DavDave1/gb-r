use crate::{debugger::debugger::AsmState, gbr::cpu::CpuState};

const COL_MIN_WIDTH: f32 = 20.0;
// const COL_MAX_WIDTH: f32 = 200.0;

pub fn show(asm: &AsmState, cpu: &CpuState, ui: &mut egui::Ui) {
    egui::ScrollArea::vertical()
        .auto_shrink([true, false])
        .show(ui, |ui| {
            egui::Grid::new("Asm View")
                .min_col_width(COL_MIN_WIDTH)
                // .max_col_width(COL_MAX_WIDTH)
                .striped(true)
                .show(ui, |ui| {
                    for (pc, instruction) in asm.iter() {
                        let cursor = if *pc == cpu.pc { "> " } else { "" };

                        let label = match instruction {
                            Some(instr) => format!("{}{:#06X}: {}", cursor, pc, instr),
                            None => format!("{}{:#06X}: Unknonwn instruction", cursor, pc),
                        };
                        ui.label(label);
                        ui.end_row();
                    }
                });
        });
}
