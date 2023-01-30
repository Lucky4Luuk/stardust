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
        ui.columns(2, |columns| {
            columns[0].label("Name");
            columns[1].menu_button("+", |ui| {
                ui.button("Entity");
            });
        });
        ui.separator();
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
