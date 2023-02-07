use std::collections::HashMap;
use std::rc::Rc;
use std::path::{Path, PathBuf};
use std::fs::{File, DirEntry};
use std::io::{Read, Write};
use egui_extras::RetainedImage;
use anyhow::Error;
use thiserror::Error;

use stardust_sdvx::Model;

pub struct Vfs {
    root: PathBuf,
}

impl Vfs {
    pub fn new<S: Into<String>>(path: S) -> Self {
        Self {
            root: path.into().into(),
        }
    }

    pub fn get_full_path<S: AsRef<Path>>(&self, path: S) -> PathBuf {
        let path = path.as_ref();
        if path.starts_with(&self.root) {
            path.to_owned()
        } else {
            let mut full_path = self.root.clone();
            full_path.push(path);
            full_path
        }
    }

    pub fn open_file<S: AsRef<Path>>(&self, path: S) -> std::io::Result<File> {
        File::open(self.get_full_path(path))
    }

    pub fn create_file<S: AsRef<Path>>(&self, path: S) -> std::io::Result<File> {
        File::create(self.get_full_path(path))
    }

    pub fn read_dir<S: AsRef<Path>>(&self, path: S) -> std::io::Result<Vec<DirEntry>> {
        let mut results = Vec::new();
        for item in std::fs::read_dir(self.get_full_path(path))? {
            match item {
                Err(_) => error!("VFS couldn't read item from directory: {:?}", item),
                Ok(item) => results.push(item),
            }
        }
        Ok(results)
    }
}

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
    // pub vfs: AltrootFS,
    pub vfs: Vfs,

    pub request_refresh: bool,
    pub requested_resources: Vec<PathBuf>,
    request_overwrite: bool,

    pub read_errors: HashMap<PathBuf, Rc<Error>>,
    pub resource_info: HashMap<PathBuf, String>,
    pub models: HashMap<PathBuf, Rc<Model>>,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            filesystem: ResourcesFilesystem::new(),
            // vfs: AltrootFS::new(VfsPath::new(PhysicalFS::new("gamedata"))),
            vfs: Vfs::new("gamedata"),

            request_refresh: true,
            requested_resources: Vec::new(),
            request_overwrite: true,

            read_errors: HashMap::new(),
            resource_info: HashMap::new(),
            models: HashMap::new(),
        }
    }

    pub fn load_model(&mut self, path: PathBuf, ctx: &foxtail::Context, world: &mut stardust_world::World) {
        if self.models.contains_key(&path) && self.request_overwrite == false { return; } // Already loaded
        match self.vfs.open_file(&path) {
            Err(e) => { self.read_errors.insert(path.clone(), Rc::new(e.into())); },
            Ok(mut file) => {
                let mut bytes = Vec::new();
                if let Err(e) = file.read_to_end(&mut bytes) {
                    self.read_errors.insert(path.clone(), Rc::new(e.into()));
                    return;
                }
                drop(file);

                match Model::from_bytes(&bytes) {
                    Err(e) => {
                        self.read_errors.insert(path.clone(), Rc::new(e.into()));
                        return;
                    },
                    Ok(model) => {
                        self.models.insert(path.clone(), Rc::new(model));
                    }
                }
            },
        }

        if let Ok(model) = self.fetch_model(&path) {
            let gpu_model = stardust_world::GpuModel::from_model(ctx, path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or(String::from("UNKNOWN")), model);
            let gpu_model = std::sync::Arc::new(gpu_model);
            world.register_model(std::sync::Arc::clone(&gpu_model));
        }
    }

    pub fn load_resource(&mut self, path: PathBuf, ctx: &foxtail::Context, world: &mut stardust_world::World) {
        let extension = path.extension().map(|s| s.to_str().unwrap_or("")).unwrap_or("");
        match extension {
            "sdvx" => self.load_model(path, ctx, world),
            "vox" => {
                // TODO: Check if path already exists? Also use overwrite here
                match self.vfs.open_file(&path) {
                    Err(e) => { self.read_errors.insert(path.clone(), Rc::new(e.into())); },
                    Ok(mut file) => {
                        let mut bytes = Vec::new();
                        if let Err(e) = file.read_to_end(&mut bytes) {
                            self.read_errors.insert(path.clone(), Rc::new(e.into()));
                            return;
                        }
                        drop(file);

                        match stardust_magica_voxel::MagicaVoxelModel::from_bytes(&bytes) {
                            Ok(mv_model) => {
                                let sdvx_model = mv_model.to_sdvx();
                                let mut sdvx_path = path.clone();
                                sdvx_path.set_extension("sdvx");
                                match sdvx_model.to_bytes() {
                                    Err(e) => { self.read_errors.insert(sdvx_path.clone(), Rc::new(e.into())); },
                                    Ok(bytes) => match self.vfs.create_file(&sdvx_path) {
                                        Err(e) => { self.read_errors.insert(sdvx_path.clone(), Rc::new(e.into())); },
                                        Ok(mut file) => {
                                            if let Err(e) = file.write_all(&bytes) {
                                                error!("Error writing path: {}", e);
                                            }
                                            self.load_resource(sdvx_path.clone(), ctx, world);
                                            self.resource_info.insert(path.clone(), format!("File was used to generate {}", sdvx_path.display()));
                                        },
                                    },
                                }
                            },
                            Err(e) => { self.read_errors.insert(path.clone(), e.into()); },
                        }
                    }
                }
            },
            _ => { self.read_errors.insert(path.clone(), Rc::new(ResourceManagerError::UnknownFileType.into())); },
        }
    }

    fn gather_resources_internal(&mut self, path: PathBuf) {
        debug!("path: {:?}", path);
        match self.vfs.read_dir(&path) {
            Ok(items) => {
                for item in items {
                    let item_path = item.path();
                    if item_path.is_dir() {
                        self.gather_resources_internal(item_path);
                    } else {
                        self.requested_resources.push(item_path);
                    }
                }
            },
            Err(e) => panic!("{}\npath: {:?}", e, path),
        }
    }

    pub fn load_next_resource(&mut self, ctx: &foxtail::Context, world: &mut stardust_world::World) {
        if self.requested_resources.len() > 0 {
            self.load_resource(self.requested_resources[0].clone(), ctx, world);
            self.requested_resources.remove(0);
        }
    }

    pub fn request_all_resources(&mut self, overwrite: bool) -> usize {
        self.request_overwrite = overwrite;
        self.requested_resources = Vec::new();
        self.gather_resources_internal(PathBuf::new());
        self.requested_resources.len()
    }

    pub fn fetch_model<P: AsRef<Path>>(&self, path: P) -> Result<&Rc<Model>, Rc<Error>> {
        let path = path.as_ref();
        if let Some(model) = self.models.get(path) {
            Ok(model)
        } else {
            Err(self.read_errors.get(path).map(|e| Rc::clone(e)).unwrap_or(Rc::new(ResourceManagerError::PathDoesntExist.into())))
        }
    }

    pub fn fetch_resource(&self, path: PathBuf) -> ManagedResource {
        if let Ok(model) = self.fetch_model(&path) {
            return ManagedResource::Model(Rc::clone(model));
        }
        if let Some(info) = self.resource_info.get(&path) {
            return ManagedResource::Info(info.clone());
        }
        ManagedResource::Error(self.read_errors.get(&path).map(|e| Rc::clone(e)).unwrap_or(Rc::new(ResourceManagerError::PathDoesntExist.into())))
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
