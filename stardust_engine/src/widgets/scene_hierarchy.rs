use stardust_ecs::prelude::*;

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
        ui.columns(2, |columns| {
            columns[0].label("Name");
            columns[1].menu_button("+", |ui| {
                if ui.button("Entity").clicked() {
                    engine.current_scene.create_entity("Entity", |entity| {
                        entity.with(CompTransform::new())
                    });
                }
            });
        });
        ui.separator();
        egui::Grid::new("scene_hierarchy").striped(true).num_columns(1).show(ui, |ui| {
            if let Some(entity) = engine.selected_entity {
                if engine.current_scene.entity_is_alive(entity) == false {
                    engine.selected_entity = None;
                }
            }
            for entity_info in engine.current_scene.entity_list() {
                ui.horizontal(|ui| {
                    match entity_info.kind {
                        EntityType::Entity(entity) => {
                            ui.label("E");
                            ui.selectable_value(&mut engine.selected_entity, Some(entity), &entity_info.name);
                        },
                        _ => {},
                    }

                    ui.with_layout(
                        egui::Layout::left_to_right(egui::Align::LEFT)
                            .with_main_justify(true)
                            .with_main_align(egui::Align::LEFT),
                        |ui| {
                            ui.label("");
                        }
                    );
                });
                ui.end_row();
            }
        });
    }
}
