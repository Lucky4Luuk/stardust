use crate::math::*;

pub struct Camera {
    pub pos: Vec3,
    pub rotation: Quat,

    pub fov_rad_y: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            pos: vec3(0.0, 0.0, 0.0),
            rotation: Quat::IDENTITY,
            fov_rad_y: 60.0 / 180.0 * std::f32::consts::PI,
        }
    }
}

impl Camera {
    pub fn set_fov_deg(&mut self, fov_deg_y: f32) {
        self.fov_rad_y = fov_deg_y / 180.0 * std::f32::consts::PI;
    }

    pub fn set_fov_rad(&mut self, fov_rad_y: f32) {
        self.fov_rad_y = fov_rad_y;
    }

    pub fn matrix_view(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.rotation, self.pos)
    }

    pub fn matrix_projection(&self, aspect_ratio: f32) -> Mat4 {
        Mat4::perspective_rh_gl(self.fov_rad_y, aspect_ratio, 0.02, 512.0)
    }

    pub fn matrix_invprojview(&self, aspect_ratio: f32) -> Mat4 {
        (self.matrix_projection(aspect_ratio) * self.matrix_view()).inverse()
    }
}
