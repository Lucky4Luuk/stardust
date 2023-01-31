use std::collections::HashMap;
use std::rc::Rc;
use egui_extras::RetainedImage;
use vfs::*;
use anyhow::Error;
use thiserror::Error;

use stardust_sdvx::{RawModel, Model};

#[derive(Debug, Error)]
pub enum ResourceManagerError {
    #[error("Unknown file type!")]
    UnknownFileType,
    #[error("Path does not exist or wasn't loaded!")]
    PathDoesntExist,
}

#[derive(Debug)]
pub enum ManagedResource {
    Error(Rc<Error>),
    Info(String),
    Model(Rc<Model>),
}

pub struct ResourceManager {
    pub filesystem: ResourcesFilesystem,
    pub vfs: AltrootFS,

    pub request_refresh: bool,
    pub requested_resources: Vec<String>,
    request_overwrite: bool,

    pub read_errors: HashMap<String, Rc<Error>>,
    pub resource_info: HashMap<String, String>,
    pub models: HashMap<String, Rc<Model>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            filesystem: ResourcesFilesystem::new(),
            vfs: AltrootFS::new(VfsPath::new(PhysicalFS::new("gamedata"))),

            request_refresh: true,
            requested_resources: Vec::new(),
            request_overwrite: false,

            read_errors: HashMap::new(),
            resource_info: HashMap::new(),
            models: HashMap::new(),
        }
    }

    pub fn load_model(&mut self, path: &str) {
        if self.models.contains_key(path) && self.request_overwrite == false { return; } // Already loaded
        match self.vfs.open_file(path) {
            Err(e) => { self.read_errors.insert(path.to_string(), Rc::new(e.into())); },
            Ok(mut file) => {
                let mut bytes = Vec::new();
                if let Err(e) = file.read_to_end(&mut bytes) {
                    self.read_errors.insert(path.to_string(), Rc::new(e.into()));
                    return;
                }
                drop(file);

                match RawModel::from_bytes(&bytes) {
                    Err(e) => { self.read_errors.insert(path.to_string(), Rc::new(e.into())); },
                    Ok(raw_model) => {
                        let model = Model::from_raw(raw_model);
                        self.models.insert(path.to_string(), Rc::new(model));
                    }
                }
            },
        }
    }

    pub fn load_resource(&mut self, path: &str) {
        let mut extension = path.rsplit("/").next().unwrap_or("").rsplit(".").next().unwrap_or("");
        let mut path_wo_ext = path.to_string();
        if extension == path.rsplit("/").next().unwrap_or("") {
            extension = "";
        } else {
            for _ in 0..extension.len()+1 {
                path_wo_ext.pop();
            }
        }
        match extension {
            "sdvx" => self.load_model(path),
            "vox" => {
                // TODO: Check if path already exists? Also use overwrite here
                match self.vfs.open_file(path) {
                    Err(e) => { self.read_errors.insert(path.to_string(), Rc::new(e.into())); },
                    Ok(mut file) => {
                        let mut bytes = Vec::new();
                        if let Err(e) = file.read_to_end(&mut bytes) {
                            self.read_errors.insert(path.to_string(), Rc::new(e.into()));
                            return;
                        }
                        drop(file);

                        match stardust_magica_voxel::MagicaVoxelModel::from_bytes(&bytes) {
                            Ok(mv_model) => {
                                let sdvx_model = mv_model.to_sdvx();
                                let sdvx_path = format!("{}.sdvx", path_wo_ext);
                                match self.vfs.create_file(&sdvx_path) {
                                    Err(e) => { self.read_errors.insert(sdvx_path.to_string(), Rc::new(e.into())); },
                                    Ok(mut file) => {
                                        if let Err(e) = file.write_all(&sdvx_model.to_bytes()) {
                                            error!("Error writing path: {}", e);
                                        }
                                        self.load_resource(&sdvx_path);
                                        self.resource_info.insert(path.to_string(), format!("File was used to generate {}", sdvx_path));
                                    },
                                }
                            },
                            Err(e) => { self.read_errors.insert(path.to_string(), e.into()); },
                        }
                    }
                }
            },
            _ => { self.read_errors.insert(path.to_string(), Rc::new(ResourceManagerError::UnknownFileType.into())); },
        }
    }

    fn gather_resources_internal(&mut self, path: &str) {
        if let Ok(items) = self.vfs.read_dir(path) {
            for item in items {
                if let Ok(metadata) = self.vfs.metadata(&format!("{}/{}", path, item)) {
                    match metadata.file_type {
                        VfsFileType::Directory => self.gather_resources_internal(&format!("{}/{}", path, item)),
                        VfsFileType::File => self.requested_resources.push(format!("{}/{}", path, item)),
                    }
                }
            }
        }
    }

    pub fn load_next_resource(&mut self) {
        if self.requested_resources.len() > 0 {
            self.load_resource(&self.requested_resources[0].clone());
            self.requested_resources.remove(0);
        } else {
            // Just to make sure
            self.request_overwrite = false;
        }
    }

    pub fn request_all_resources(&mut self, overwrite: bool) -> usize {
        self.request_overwrite = overwrite;
        self.requested_resources = Vec::new();
        self.gather_resources_internal(".");
        self.requested_resources.iter_mut().for_each(|fp| {
            if fp.starts_with("./") {
                *fp = fp[2..].to_string();
            }
            if fp.starts_with("/") {
                *fp = fp[1..].to_string();
            }
        });
        self.requested_resources.len()
    }

    pub fn fetch_model(&self, path: &str) -> Result<&Rc<Model>, Rc<Error>> {
        if let Some(model) = self.models.get(path) {
            Ok(model)
        } else {
            Err(self.read_errors.get(path).map(|e| Rc::clone(e)).unwrap_or(Rc::new(ResourceManagerError::PathDoesntExist.into())))
        }
    }

    pub fn fetch_resource(&self, path: &str) -> ManagedResource {
        if let Ok(model) = self.fetch_model(path) {
            return ManagedResource::Model(Rc::clone(model));
        }
        if let Some(info) = self.resource_info.get(path) {
            return ManagedResource::Info(info.clone());
        }
        ManagedResource::Error(self.read_errors.get(path).map(|e| Rc::clone(e)).unwrap_or(Rc::new(ResourceManagerError::PathDoesntExist.into())))
    }
}

pub struct ResourcesFilesystem {
    pub folder_icon: RetainedImage,
    pub unknown_icon: RetainedImage,
    pub scene_icon: RetainedImage,
    pub sound_icon: RetainedImage,
    pub image_icon: RetainedImage,
    pub voxel_model_icon: RetainedImage,
}

impl ResourcesFilesystem {
    pub fn new() -> Self {
        Self {
            folder_icon: RetainedImage::from_image_bytes(
                "icon_folder.png",
                include_bytes!("../resources/icons/icon_folder.png"),
            ).expect("Failed to load folder icon!"),
            unknown_icon: RetainedImage::from_image_bytes(
                "icon_file_unknown.png",
                include_bytes!("../resources/icons/icon_file_unknown.png"),
            ).expect("Failed to load folder icon!"),
            scene_icon: RetainedImage::from_image_bytes(
                "icon_file_scene.png",
                include_bytes!("../resources/icons/icon_file_scene.png"),
            ).expect("Failed to load folder icon!"),
            sound_icon: RetainedImage::from_image_bytes(
                "icon_file_sound.png",
                include_bytes!("../resources/icons/icon_file_sound.png"),
            ).expect("Failed to load folder icon!"),
            image_icon: RetainedImage::from_image_bytes(
                "icon_file_image.png",
                include_bytes!("../resources/icons/icon_file_image.png"),
            ).expect("Failed to load folder icon!"),
            voxel_model_icon: RetainedImage::from_image_bytes(
                "icon_file_voxel_model.png",
                include_bytes!("../resources/icons/icon_file_voxel_model.png"),
            ).expect("Failed to load folder icon!"),
        }
    }

    pub fn file_icon_from_extension(&self, extension: &str) -> &RetainedImage {
        match extension {
            "png" | "jpeg" => &self.image_icon,
            "sdvx" | "vox" => &self.voxel_model_icon,
            "wav" | "ogg" => &self.sound_icon,
            _ => &self.unknown_icon
        }
    }
}
