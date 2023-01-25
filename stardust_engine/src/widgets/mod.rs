mod vfs_browser;
pub use vfs_browser::*;

mod console;
pub use console::*;

pub trait Widget {
    fn title(&self) -> String;
    fn draw(&mut self, ui: &mut egui::Ui, engine: &mut crate::EngineInternals);
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DockLoc {
    Left,
    Right,
    Top,
    Bottom
}

struct DockedWidget {
    widget: Box<dyn Widget>,
    dock: DockLoc,
}

pub struct WidgetContainer {
    left_docked_widgets: Vec<DockedWidget>,
    right_docked_widgets: Vec<DockedWidget>,
    bottom_docked_widgets: Vec<DockedWidget>,
}

impl WidgetContainer {
    pub fn new() -> Self {
        Self {
            left_docked_widgets: Vec::new(),
            right_docked_widgets: Vec::new(),
            bottom_docked_widgets: Vec::new(),
        }
    }

    pub fn add_docked(&mut self, widget: Box<dyn Widget>, loc: DockLoc) {
        let docked_widget = DockedWidget {
            widget: widget,
            dock: loc,
        };
        match loc {
            DockLoc::Left => self.left_docked_widgets.push(docked_widget),
            DockLoc::Right => self.right_docked_widgets.push(docked_widget),
            DockLoc::Bottom => self.bottom_docked_widgets.push(docked_widget),
            _ => unimplemented!()
        }
    }

    pub fn draw_docked(&mut self, ctx: &egui::Context, engine: &mut crate::EngineInternals) {
        if self.left_docked_widgets.len() > 0 {
            egui::SidePanel::left("docked_left").show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("docked_left_grid").num_columns(1).show(ui, |ui| {
                        for docked_widget in &mut self.left_docked_widgets {
                            ui.vertical(|ui| {
                                ui.heading(docked_widget.widget.title());
                                ui.separator();
                                docked_widget.widget.draw(ui, engine);
                                ui.separator();
                            });
                            ui.end_row();
                        }
                    });
                });
            });
        }

        if self.right_docked_widgets.len() > 0 {
            egui::SidePanel::right("docked_right").show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("docked_right_grid").num_columns(1).show(ui, |ui| {
                        for docked_widget in &mut self.right_docked_widgets {
                            ui.vertical(|ui| {
                                ui.heading(docked_widget.widget.title());
                                ui.separator();
                                docked_widget.widget.draw(ui, engine);
                                ui.separator();
                            });
                            ui.end_row();
                        }
                    });
                });
            });
        }

        if self.bottom_docked_widgets.len() > 0 {
            egui::TopBottomPanel::bottom("docked_bottom").show(ctx, |ui| {
                egui::Grid::new("docked_bottom_grid").num_columns(self.bottom_docked_widgets.len()).show(ui, |ui| {
                    for docked_widget in &mut self.bottom_docked_widgets {
                        ui.horizontal(|ui| {
                            ui.heading(docked_widget.widget.title());
                            ui.separator();
                            docked_widget.widget.draw(ui, engine);
                            ui.separator();
                        });
                        ui.end_row();
                    }
                });
            });
        }
    }
}
