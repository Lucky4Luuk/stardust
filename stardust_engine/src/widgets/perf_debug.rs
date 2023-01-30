pub struct PerfDebug;

impl super::Widget for PerfDebug {
    fn title(&self) -> String {
        String::from("Debug")
    }

    fn draw(&mut self, ui: &mut egui::Ui, engine: &mut crate::EngineInternals) {
        ui.heading("Debug");
        ui.label(&format!("fps: {}", 1.0 / engine.delta_s));
        ui.label(&format!("ms: {}", engine.delta_s * 1000.0));
        ui.label(&format!("render resolution: {:?}", engine.render_size));
        ui.label(&format!("cam_pos: {:?}", engine.camera.pos));
        ui.label(&format!("bricks used: {}/{}", engine.world.bricks_used, stardust_world::BRICK_POOL_SIZE));
        ui.label(&format!("layer0 used: {}/{}", engine.world.layer0_used, stardust_world::LAYER0_POOL_SIZE));
    }
}
