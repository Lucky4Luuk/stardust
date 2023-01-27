pub struct SceneHierachy {

}

impl SceneHierachy {
    pub fn new() -> Self {
        Self {

        }
    }
}

impl super::Widget for SceneHierachy {
    fn title(&self) -> String {
        String::from("Scene hierarchy")
    }

    fn draw(&mut self, ui: &mut egui::Ui, engine: &mut crate::EngineInternals) {
        //striped grid
    }
}
