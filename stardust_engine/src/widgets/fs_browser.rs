use std::path::{Path, PathBuf};

fn filename_from_path<P: AsRef<Path>>(item: P) -> Option<String> {
    let item = item.as_ref();
    item.file_name().map(|s| s.to_string_lossy().to_string())
}

fn extension_from_path<P: AsRef<Path>>(item: P) -> String {
    let item = item.as_ref();
    item.extension().map(|s| s.to_str().unwrap_or("").to_string()).unwrap_or(String::new())
}

pub struct FsBrowser {
    active_folder: PathBuf,

    folders: Vec<PathBuf>,
    files: Vec<PathBuf>,

    request_refresh: bool,
}

impl FsBrowser {
    pub fn new() -> Self {
        Self {
            active_folder: PathBuf::new(),

            folders: Vec::new(),
            files: Vec::new(),

            request_refresh: true,
        }
    }

    pub fn refresh(&mut self, engine: &mut crate::EngineInternals) {
        // Gather a list of items in the directory
        self.folders = Vec::new();
        self.files = Vec::new();
        if let Ok(items) = engine.resources.vfs.read_dir(&self.active_folder) {
            for item in items {
                let item_path = item.path();
                if item_path.is_dir() {
                    self.folders.push(item_path);
                } else {
                    self.files.push(item_path);
                }
            }
        }
        self.request_refresh = false;
    }

    pub fn browse_local(&mut self, local_path: PathBuf) {
        if let Some(fp) = filename_from_path(local_path) {
            self.active_folder.push(fp);
            self.request_refresh = true;
        }
    }
}

impl super::Widget for FsBrowser {
    fn title(&self) -> String {
        String::from("File Browser")
    }

    fn resizable(&self) -> bool {
        false
    }

    fn draw(&mut self, ctx: &mut super::WidgetContext, ui: &mut egui::Ui, engine: &mut crate::EngineInternals) {
        if self.request_refresh { self.refresh(engine); }

        // if self.active_folder.parent().map(|parent| parent.parent().is_some()).unwrap_or(false) {
        if self.active_folder.parent().is_some() {
            if ui.button("../").clicked() {
                self.active_folder.pop();
                self.request_refresh = true;
            }
        }

        let mut next_folder = None;

        // Draw a list of items
        if self.folders.len() > 0 || self.files.len() > 0 {
            egui::ScrollArea::vertical().show(ui, |ui| {
                let available_width = ui.available_width();
                let cell_width = 72f32;
                let button_width = 48f32;
                let buttons = ((available_width / cell_width) as usize).max(1);
                egui::Grid::new("fs_browser_grid").num_columns(buttons).min_col_width(cell_width).show(ui, |ui| {
                    let mut i = 0;
                    let folder_tex_id = engine.resources.filesystem.folder_icon.texture_id(ui.ctx());
                    for f in &self.folders {
                        ui.vertical_centered(|ui| {
                            if ui.add(egui::ImageButton::new(folder_tex_id, (button_width, button_width))).clicked() {
                                next_folder = Some(f.clone());
                            }
                            ui.label(filename_from_path(f).unwrap());
                        });
                        i += 1;
                        if i >= buttons {
                            i = 0;
                            ui.end_row();
                        }
                    }
                    for f in &self.files {
                        ui.vertical_centered(|ui| {
                            let extension = extension_from_path(f);
                            let filename = filename_from_path(f).unwrap();
                            let tex_id = engine.resources.filesystem.file_icon_from_extension(&extension).texture_id(ui.ctx());
                            let resp = ui.add(egui::ImageButton::new(tex_id, (button_width, button_width)));
                            if resp.clicked() {
                                debug!("[BUTTON] file clicked: {}", f.display());
                                let resource = engine.resources.fetch_resource(f.into());
                                ctx.add_widget(Box::new(super::ResourceInspector::new(resource, filename.clone())), super::DockLoc::Floating);
                            }
                            ui.label(filename);
                        });
                        i += 1;
                        if i >= buttons {
                            i = 0;
                            ui.end_row();
                        }
                    }
                });
            });
        } else {
            ui.label("Nothing here...");
        }

        if let Some(f) = next_folder {
            self.browse_local(f);
        }

        // Draw footer
        ui.separator();
        ui.horizontal(|ui| {
            if ui.button("Load new assets").clicked() && engine.resources.requested_resources.len() == 0 {
                ctx.add_widget(Box::new(super::ResourceLoader::new(engine, false)), super::DockLoc::Floating);
            }
            if ui.button("Reload all assets").clicked() && engine.resources.requested_resources.len() == 0 {
                ctx.add_widget(Box::new(super::ResourceLoader::new(engine, true)), super::DockLoc::Floating);
            }
        });
    }
}
