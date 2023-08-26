use crate::{debugger::debugger::AsmState, gbr::cpu::CpuState};

use egui::{Label, Sense};
use egui_extras::{Column, TableBuilder};
use log::info;

// const COL_MIN_WIDTH: f32 = 20.0;
// const COL_MAX_WIDTH: f32 = 200.0;

pub fn show(asm: &AsmState, cpu: &CpuState, ui: &mut egui::Ui) {
    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

    egui::ScrollArea::horizontal()
        .auto_shrink([true, false])
        .show(ui, |ui| {
            TableBuilder::new(ui)
                .striped(true)
                .resizable(false)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::exact(10.0))
                .column(Column::exact(10.0))
                .column(Column::remainder())
                .min_scrolled_height(0.0)
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("");
                    });
                    header.col(|ui| {
                        ui.strong("");
                    });
                    header.col(|ui| {
                        ui.strong("ASM");
                    });
                })
                .body(|body| {
                    body.rows(text_height, asm.len(), |index, mut row| {
                        let (pc, instruction) = asm.iter().nth(index).as_ref().unwrap();

                        row.col(|ui| {
                            if ui
                                .add(Label::new(" ").sense(Sense::click()))
                                .double_clicked()
                            {
                                info!("Double clicked at {}", index);
                            }
                        });

                        let cursor = if *pc == cpu.pc { ">" } else { "" };

                        row.col(|ui| {
                            ui.label(cursor);
                        });

                        let instr_label = match instruction {
                            Some(instr) => format!("{:#06X}: {}", *pc, instr),
                            None => format!("{:#06X}: Unknonwn", *pc),
                        };
                        row.col(|ui| {
                            ui.label(instr_label);
                        });
                    });
                });
        });
}
