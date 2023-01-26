use std::path::{Path, PathBuf};

use vfs::*;

pub struct FsBrowser {
    active_folder: Option<PathBuf>,
}

impl FsBrowser {
    pub fn new() -> Self {
        Self {
            active_folder: None,
        }
    }
}

impl super::Widget for FsBrowser {
    fn title(&self) -> String {
        String::from("File Browser")
    }

    fn draw(&mut self, ui: &mut egui::Ui, engine: &mut crate::EngineInternals) {
        if self.active_folder.is_some() {
            if ui.button("../").clicked() {
                match self.active_folder.as_ref().unwrap().parent() {
                    Some(parent) => self.active_folder = Some(parent.to_owned()),
                    None => self.active_folder = None,
                }
            }
        }

        // Draw a list of items in the directory
        let path = self.active_folder.as_ref().map(|s| s.display().to_string()).unwrap_or(String::from("."));
        let mut folders = Vec::new();
        let mut files = Vec::new();
        if let Ok(items) = engine.vfs.read_dir(&path) {
            for item in items {
                if let Ok(metadata) = engine.vfs.metadata(&format!("{}/{}", path, item)) {
                    match metadata.file_type {
                        VfsFileType::Directory => folders.push(item),
                        _ => files.push(item),
                    }
                }
            }
        }

        if folders.len() > 0 || files.len() > 0 {
            for f in folders {
                ui.label(format!("{}/", f));
            }
            for f in files {
                ui.label(f);
            }
        } else {
            ui.label("Nothing here...");
        }
    }
}
