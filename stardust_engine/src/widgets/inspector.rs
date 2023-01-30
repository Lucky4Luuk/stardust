use std::collections::BTreeMap;

use stardust_ecs::prelude::*;

pub struct Inspector {
    current_entity: Option<Entity>,
    current_components: Option<EntityComponentInfo>,
}

impl Inspector {
    pub fn new() -> Self {
        Self {
            current_entity: None,
            current_components: None,
        }
    }

    pub fn refresh(&mut self, engine: &mut crate::EngineInternals) {
        if let Some(entity) = self.current_entity {
            self.current_components = Some(engine.current_scene.entity_component_list(entity));
        } else {
            self.current_components = None;
        }
    }
}

impl super::Widget for Inspector {
    fn title(&self) -> String {
        String::from("Inspector")
    }

    fn draw(&mut self, ui: &mut egui::Ui, engine: &mut crate::EngineInternals) {
        if engine.selected_entity != self.current_entity {
            self.current_entity = engine.selected_entity;
            self.refresh(engine);
        }

        if let Some(entity) = self.current_entity {
            if let Some(comp_info) = &mut self.current_components {
                let mut dirty = false;

                // ui.label(egui::RichText::new("Name").strong());
                // dirty = dirty || draw_component_name(ui, engine, &mut comp_info.name_component);
                // if let Some(ctransform) = &mut comp_info.transform_component {
                //     ui.separator();
                //     ui.label(egui::RichText::new("Transform").strong());
                //     dirty = dirty || draw_component_transform(ui, engine, ctransform);
                // }
                // if let Some(cmodel) = &mut comp_info.model_component {
                //     ui.separator();
                //     ui.label(egui::RichText::new("Model").strong());
                //     dirty = dirty || draw_component_model(ui, engine, cmodel);
                // }
                dirty = dirty || draw_generic_component(ui, engine, "Name", comp_info.name_component.fields());
                if let Some(ctransform) = &mut comp_info.transform_component {
                    dirty = dirty || draw_generic_component(ui, engine, "Transform", ctransform.fields());
                }
                if let Some(cmodel) = &mut comp_info.model_component {
                    dirty = dirty || draw_generic_component(ui, engine, "Model", cmodel.fields());
                }

                ui.separator();
                ui.menu_button("Add component", |menu| {
                    if menu.button("Transform").clicked() {
                        comp_info.transform_component = Some(CompTransform::new());
                        dirty = true;
                    }

                    if menu.button("Model").clicked() {
                        comp_info.model_component = Some(CompModel::new());
                        dirty = true;
                    }
                });

                if dirty {
                    engine.current_scene.entity_upload_component_list(entity, comp_info.clone());
                    self.refresh(engine);
                }
            }
        }
    }
}

fn draw_generic_component<S: Into<String>>(ui: &mut egui::Ui, engine: &mut crate::EngineInternals, name: S, fields: BTreeMap<String, Value>) -> bool {
    let name = name.into();

    let mut dirty = false;

    ui.label(egui::RichText::new(&name).strong());
    egui::Grid::new(format!("inspector_comp_generic_{}", &name)).num_columns(2).show(ui, |ui| {
        for (k, v) in fields {
            ui.label(k);
            let responses = match v {
                Value::String(s) => vec![ui.text_edit_singleline(s)],
                Value::Float(f) => vec![ui.add(egui::DragValue::new(f))],
                Value::Vec2(x, y) => {
                    let mut responses = Vec::new();
                    ui.columns(2, |columns| {
                        responses.push(columns[0].add(egui::DragValue::new(x)));
                        responses.push(columns[1].add(egui::DragValue::new(y)));
                    });
                    responses
                },
                Value::Vec3(x, y, z) => {
                    let mut responses = Vec::new();
                    ui.columns(3, |columns| {
                        responses.push(columns[0].add(egui::DragValue::new(x)));
                        responses.push(columns[1].add(egui::DragValue::new(y)));
                        responses.push(columns[2].add(egui::DragValue::new(z)));
                    });
                    responses
                },
                Value::Vec4(x, y, z, w) => {
                    let mut responses = Vec::new();
                    ui.columns(4, |columns| {
                        responses.push(columns[0].add(egui::DragValue::new(x)));
                        responses.push(columns[1].add(egui::DragValue::new(y)));
                        responses.push(columns[2].add(egui::DragValue::new(z)));
                        responses.push(columns[3].add(egui::DragValue::new(w)));
                    });
                    responses
                },
                _ => unimplemented!(),
            };

            for resp in responses {
                dirty = dirty || resp.lost_focus() || resp.changed();
            }

            ui.end_row();
        }
    });

    dirty
}
