use std::path::{Path, PathBuf};

use vfs::*;

pub struct FsBrowser {
    active_folder: Option<PathBuf>,

    folders: Vec<String>,
    files: Vec<String>,

    request_refresh: bool,
}

impl FsBrowser {
    pub fn new() -> Self {
        Self {
            active_folder: None,

            folders: Vec::new(),
            files: Vec::new(),

            request_refresh: true,
        }
    }

    pub fn refresh(&mut self, engine: &mut crate::EngineInternals) {
        // Gather a list of items in the directory
        self.folders = Vec::new();
        self.files = Vec::new();
        let path = self.active_folder.as_ref().map(|s| s.display().to_string()).unwrap_or(String::from("."));
        if let Ok(items) = engine.vfs.read_dir(&path) {
            for item in items {
                if let Ok(metadata) = engine.vfs.metadata(&format!("{}/{}", path, item)) {
                    match metadata.file_type {
                        VfsFileType::Directory => self.folders.push(item),
                        _ => self.files.push(item),
                    }
                }
            }
        }
        self.request_refresh = false;
    }

    pub fn browse_local(&mut self, local_path: String) {
        let mut current_path = match &self.active_folder {
            Some(path) => path.clone(),
            None => PathBuf::new(),
        };
        current_path.push(local_path);
        self.active_folder = Some(current_path);
        self.request_refresh = true;
    }
}

impl super::Widget for FsBrowser {
    fn title(&self) -> String {
        String::from("File Browser")
    }

    fn resizable(&self) -> bool {
        false
    }

    fn draw(&mut self, ui: &mut egui::Ui, engine: &mut crate::EngineInternals) {
        if self.request_refresh { self.refresh(engine); }

        if self.active_folder.is_some() {
            if ui.button("../").clicked() {
                match self.active_folder.as_ref().unwrap().parent() {
                    Some(parent) => self.active_folder = Some(parent.to_owned()),
                    None => self.active_folder = None,
                }
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
                            ui.label(f);
                        });
                        i += 1;
                        if i >= buttons {
                            i = 0;
                            ui.end_row();
                        }
                    }
                    for f in &self.files {
                        ui.vertical_centered(|ui| {
                            let extension = f.rsplit(".").next().unwrap_or("");
                            let tex_id = engine.resources.filesystem.file_icon_from_extension(extension).texture_id(ui.ctx());
                            if ui.add(egui::ImageButton::new(tex_id, (button_width, button_width))).clicked() {
                                // next_folder = Some(f.clone());
                                debug!("[BUTTON] file clicked: {}", f);
                            }
                            ui.label(f);
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
    }
}
