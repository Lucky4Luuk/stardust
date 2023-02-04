use std::sync::Arc;
use stardust_ecs::prelude::*;

pub struct ModelSelector {
    entity: Entity,
    comp_name: String,
    field_name: String,

    close: bool,
}

impl ModelSelector {
    pub fn new(entity: Entity, comp_name: String, field_name: String) -> Self {
        Self {
            entity,
            comp_name,
            field_name,

            close: false,
        }
    }
}

impl super::Widget for ModelSelector {
    fn title(&self) -> String { String::from("Model selector") }
    fn resizable(&self) -> bool { false }
    fn update_open_status(&self, open: &mut bool) {
        if self.close {
            *open = false;
        }
    }

    fn draw(&mut self, ctx: &mut super::WidgetContext, ui: &mut egui::Ui, engine: &mut crate::EngineInternals) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::Grid::new("model_selector_grid").num_columns(2).show(ui, |ui| {
                for (i, model) in engine.world.gpu_models.iter().enumerate() {
                    let mut name = model.name.clone();
                    name = name.split("/").last().unwrap_or(&name).to_string();
                    if name.len() > 8 {
                        name = name[..5].to_string();
                        name.push_str("...");
                    }
                    if ui.button(name).clicked() {
                        let mut comp_info = engine.current_scene.entity_component_list(self.entity);
                        if let Some(comp) = comp_info.components.get_mut(&self.comp_name) {
                            comp.set_field(&self.field_name, ValueOwned::ModelReference(Some(Arc::clone(model)))).unwrap();
                            engine.current_scene.entity_upload_component_list(self.entity, &comp_info);
                        }
                        self.close = true;
                    }
                    if i % 2 == 1 {
                        ui.end_row();
                    }
                }
            });
        });
    }
}
