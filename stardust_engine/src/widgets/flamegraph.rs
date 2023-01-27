pub struct Flamegraph {

}

impl Flamegraph {
    pub fn new() -> Self {
        puffin::set_scopes_on(true);
        Self {

        }
    }
}

impl Drop for Flamegraph {
    fn drop(&mut self) {
        puffin::set_scopes_on(false);
    }
}

impl super::Widget for Flamegraph {
    fn title(&self) -> String {
        String::from("Flamegraph")
    }

    fn draw(&mut self, ui: &mut egui::Ui, engine: &mut crate::EngineInternals) {
        puffin_egui::profiler_ui(ui);
    }
}
