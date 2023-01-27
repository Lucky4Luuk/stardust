use egui_extras::RetainedImage;

pub struct ResourceManager {
    pub filesystem: ResourcesFilesystem,
}

impl ResourceManager {
    pub fn new() -> Self {
        Self {
            filesystem: ResourcesFilesystem::new(),
        }
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
