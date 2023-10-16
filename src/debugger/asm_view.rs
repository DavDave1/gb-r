use std::{collections::HashSet, sync::mpsc::Sender};

use crate::gbr::{
    cpu::CpuState,
    game_boy::{DebugEvent, GbrEvent},
};

use egui::{Label, Sense};
use egui_extras::{Column, TableBuilder};

use super::debugger::AsmState;

pub fn show(
    ev_sender: &Sender<GbrEvent>,
    asm: &AsmState,
    cpu: &CpuState,
    breakpoints: &mut HashSet<u16>,
    ui: &mut egui::Ui,
) {
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
                                    ev_sender
                                        .send(GbrEvent::Debug(DebugEvent::SetBreakpoint(*pc)))
                                        .unwrap();
                                }
                            } else {
                                if ui
                                    .add(Label::new(" ").sense(Sense::click()))
                                    .double_clicked()
                                {
                                    breakpoints.insert(*pc);
                                    ev_sender
                                        .send(GbrEvent::Debug(DebugEvent::ClearBreakpoint(*pc)))
                                        .unwrap();
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
