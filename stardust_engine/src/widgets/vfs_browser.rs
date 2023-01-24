pub struct VfsBrowser {

}

impl VfsBrowser {
    pub fn new() -> Self {
        Self {

        }
    }
}

impl super::Widget for VfsBrowser {
    fn title(&self) -> String {
        String::from("File Browser")
    }

    fn draw(&mut self, ui: &mut egui::Ui, engine: &mut crate::EngineInternals) {
        ui.label("Nothing to see here...");
    }
}
