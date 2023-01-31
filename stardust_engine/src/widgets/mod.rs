mod custom;
pub use custom::*;

mod fs_browser;
pub use fs_browser::*;

mod console;
pub use console::*;

mod flamegraph;
pub use flamegraph::*;

mod scene_hierarchy;
pub use scene_hierarchy::*;

mod perf_debug;
pub use perf_debug::*;

mod inspector;
pub use inspector::*;

mod resource_loader;
pub use resource_loader::*;

mod resource_inspector;
pub use resource_inspector::*;

pub trait Widget {
    fn title(&self) -> String;
    fn resizable(&self) -> bool { true }
    fn closable(&self) -> bool { true }
    fn update_open_status(&self, _open: &mut bool) {}
    fn draw(&mut self, ctx: &mut WidgetContext, ui: &mut egui::Ui, engine: &mut crate::EngineInternals);
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DockLoc {
    Left,
    Right,
    Top,
    Bottom,

    Floating
}

struct DockedWidget {
    widget: Box<dyn Widget>,
    dock: DockLoc,
}

struct FloatingWidget {
    widget: Box<dyn Widget>,
    open: bool,
}

pub struct WidgetContext {
    requested_widgets: Vec<(Box<dyn Widget>, DockLoc)>,
}

impl WidgetContext {
    fn new() -> Self {
        Self {
            requested_widgets: Vec::new(),
        }
    }

    pub fn add_widget(&mut self, widget: Box<dyn Widget>, loc: DockLoc) {
        self.requested_widgets.push((widget, loc));
    }
}

pub struct WidgetManager {
    left_docked_widgets: Vec<DockedWidget>,
    right_docked_widgets: Vec<DockedWidget>,

    bottom_docked_widgets: Vec<DockedWidget>,
    active_bottom_docked_widget: usize,

    // These will be stacked vertically, similar to those old IE ad bars and search bars lol
    top_docked_widgets: Vec<DockedWidget>,

    floating_widgets: Vec<FloatingWidget>,
}

impl WidgetManager {
    pub fn new() -> Self {
        Self {
            left_docked_widgets: Vec::new(),
            right_docked_widgets: Vec::new(),

            bottom_docked_widgets: Vec::new(),
            active_bottom_docked_widget: 0,

            top_docked_widgets: Vec::new(),

            floating_widgets: Vec::new(),
        }
    }

    pub fn add_widget(&mut self, widget: Box<dyn Widget>, loc: DockLoc) {
        if loc == DockLoc::Floating {
            // Check if widget is already open
            let title = widget.title();
            let mut existing = None;
            'search: for fw in &mut self.floating_widgets {
                if fw.widget.title() == title {
                    existing = Some(fw);
                    break 'search;
                }
            }

            if let Some(existing) = existing {
                // TODO: Focus this widget
            } else {
                let floating_widget = FloatingWidget {
                    widget: widget,
                    open: true,
                };
                self.floating_widgets.push(floating_widget);
            }
        } else {
            let docked_widget = DockedWidget {
                widget: widget,
                dock: loc,
            };
            match loc {
                DockLoc::Left => self.left_docked_widgets.push(docked_widget),
                DockLoc::Right => self.right_docked_widgets.push(docked_widget),
                DockLoc::Bottom => self.bottom_docked_widgets.push(docked_widget),
                DockLoc::Top => self.top_docked_widgets.push(docked_widget),
                _ => unimplemented!()
            }
        }
    }

    pub fn draw_floating(&mut self, ctx: &egui::Context, engine: &mut crate::EngineInternals) {
        let mut wctx = WidgetContext::new();

        for floating_widget in &mut self.floating_widgets {
            let title = floating_widget.widget.title();
            let mut builder = egui::Window::new(&title);
            floating_widget.widget.update_open_status(&mut floating_widget.open);
            if floating_widget.widget.closable() || floating_widget.open == false {
                builder = builder.open(&mut floating_widget.open);
            }
            builder.resizable(floating_widget.widget.resizable()).show(ctx, |ui| {
                floating_widget.widget.draw(&mut wctx, ui, engine);
            });
        }

        self.floating_widgets.retain(|fw| fw.open);

        for (widget, loc) in wctx.requested_widgets {
            self.add_widget(widget, loc);
        }
    }

    pub fn draw_docked(&mut self, ctx: &egui::Context, engine: &mut crate::EngineInternals) {
        let mut wctx = WidgetContext::new();

        // Menubar
        // TODO: Close windows with duplicate IDs
        egui::TopBottomPanel::top("menubar").resizable(false).show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save project...").clicked() {
                        debug!("[BUTTON] Save project...");
                    }
                });
                ui.menu_button("Widgets", |ui| {
                    if ui.button("Flamegraph").clicked() {
                        self.add_widget(Box::new(Flamegraph::new()), DockLoc::Floating);
                    }
                    if ui.button("PerfDebug").clicked() {
                        self.add_widget(Box::new(PerfDebug), DockLoc::Floating);
                    }
                });
            });
        });

        if self.left_docked_widgets.len() > 0 {
            egui::SidePanel::left("docked_left").show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("docked_left_grid").num_columns(1).show(ui, |ui| {
                        for docked_widget in &mut self.left_docked_widgets {
                            ui.vertical(|ui| {
                                ui.heading(docked_widget.widget.title());
                                ui.separator();
                                docked_widget.widget.draw(&mut wctx, ui, engine);
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
                                docked_widget.widget.draw(&mut wctx, ui, engine);
                                ui.separator();
                            });
                            ui.end_row();
                        }
                    });
                });
            });
        }

        if self.bottom_docked_widgets.len() > 0 {
            egui::TopBottomPanel::bottom("docked_bottom").resizable(true).show(ctx, |ui| {
                ui.columns(self.bottom_docked_widgets.len(), |columns| {
                    for (i, docked_widget) in self.bottom_docked_widgets.iter().enumerate() {
                        columns[i].selectable_value(&mut self.active_bottom_docked_widget, i, docked_widget.widget.title());
                    }
                });
                ui.separator();
                if let Some(widget) = self.bottom_docked_widgets.get_mut(self.active_bottom_docked_widget) {
                    widget.widget.draw(&mut wctx, ui, engine);
                } else {
                    self.active_bottom_docked_widget = 0;
                }
            });
        }

        if self.top_docked_widgets.len() > 0 {
            egui::TopBottomPanel::top("docked_top").resizable(false).show(ctx, |ui| {
                for docked_widget in &mut self.top_docked_widgets {
                    ui.horizontal_centered(|ui| {
                        docked_widget.widget.draw(&mut wctx, ui, engine);
                    });
                    ui.separator();
                }
            });
        }

        for (widget, loc) in wctx.requested_widgets {
            self.add_widget(widget, loc);
        }
    }
}
