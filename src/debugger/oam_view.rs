use crate::gbr::oam::ObjAttribute;

use egui_extras::{Column, TableBuilder};

pub fn show(oam: &[ObjAttribute], ui: &mut egui::Ui) {
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
                    body.rows(text_height, oam.len(), |index, mut row| {
                        let obj = &oam[index];

                        row.col(|ui| {
                            ui.label(format!("{}", index));
                        });

                        row.col(|ui| {
                            ui.label(format!("{}", obj.tile_index()));
                        });
                        row.col(|ui| {
                            ui.label(format!("{}, {}", obj.top(), obj.left()));
                        });
                    });
                });
        });
}
