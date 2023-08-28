use std::collections::HashSet;

use crate::{debugger::debugger::DebuggerCommand, gbr::cpu::CpuState};

use egui::{Label, Sense};
use egui_extras::{Column, TableBuilder};

use super::debugger::Debugger;

pub fn show(
    debugger: &Debugger,
    cpu: &CpuState,
    breakpoints: &mut HashSet<u16>,
    ui: &mut egui::Ui,
) {
    let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

    let asm = debugger.asm();

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
                .body(|body| {
                    body.rows(text_height, asm.len(), |index, mut row| {
                        let (pc, instruction) = asm.iter().nth(index).as_ref().unwrap();

                        row.col(|ui| {
                            if breakpoints.contains(pc) {
                                if ui
                                    .add(Label::new("*").sense(Sense::click()))
                                    .double_clicked()
                                {
                                    breakpoints.remove(pc);
                                    debugger.send_cmd(DebuggerCommand::UnsetBreakpoint(*pc));
                                }
                            } else {
                                if ui
                                    .add(Label::new(" ").sense(Sense::click()))
                                    .double_clicked()
                                {
                                    breakpoints.insert(*pc);
                                    debugger.send_cmd(DebuggerCommand::SetBreakpoint(*pc));
                                }
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
