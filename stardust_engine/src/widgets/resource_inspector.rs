use crate::resource_manager::ManagedResource;

pub struct ResourceInspector {
    resource: ManagedResource,
    name: String,
    error_buf: String,
}

impl ResourceInspector {
    pub fn new(resource: ManagedResource, name: String) -> Self {
        Self {
            resource,
            name,
            error_buf: String::new(),
        }
    }
}

impl super::Widget for ResourceInspector {
    fn title(&self) -> String {
        self.name.clone()
    }

    fn draw(&mut self, ctx: &mut super::WidgetContext, ui: &mut egui::Ui, engine: &mut crate::EngineInternals) {
        match &mut self.resource {
            ManagedResource::Error(err) => {
                self.error_buf = format!("{:#?}", err);
                ui.label("Resource could not be loaded.");
                ui.add(egui::TextEdit::multiline(&mut self.error_buf).code_editor().interactive(false));
            },
            _ => { ui.label("Filetype has no preview yet"); },
        }
    }
}
