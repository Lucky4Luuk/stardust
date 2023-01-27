use stardust_ecs::EntityInfo;

pub struct SceneHierachy {
    entity_list: Vec<EntityInfo>,
}

impl SceneHierachy {
    pub fn new() -> Self {
        Self {
            entity_list: Vec::new(),
        }
    }
}

impl super::Widget for SceneHierachy {
    fn title(&self) -> String {
        String::from("Scene hierarchy")
    }

    fn draw(&mut self, ui: &mut egui::Ui, engine: &mut crate::EngineInternals) {

        egui::Grid::new("scene_hierarchy").striped(true).num_columns(1).show(ui, |ui| {
            for entity in engine.current_scene.entity_list() {
                ui.horizontal(|ui| {
                    ui.label("E");
                    ui.label(&entity.name);
                });
                ui.end_row();
            }
        });
    }
}
