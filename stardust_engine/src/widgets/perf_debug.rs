pub struct PerfDebug;

impl super::Widget for PerfDebug {
    fn title(&self) -> String {
        String::from("PerfDebug")
    }

    fn draw(&mut self, ctx: &mut super::WidgetContext, ui: &mut egui::Ui, engine: &mut crate::EngineInternals) {
        ui.label(&format!("fps: {}", 1.0 / engine.delta_s));
        ui.label(&format!("ms: {}", engine.delta_s * 1000.0));
        ui.label(&format!("render resolution: {:?}", engine.render_size));
        ui.label(&format!("cam_pos: {:?}", engine.camera.pos));
        ui.label(&format!("voxels_queued: {}", engine.world.voxels_queued()));
    }
}
