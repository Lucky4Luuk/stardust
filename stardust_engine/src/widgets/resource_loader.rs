pub struct ResourceLoader {
    pub to_load: usize,
    pub done: bool,
}

impl ResourceLoader {
    pub fn new(engine: &mut crate::EngineInternals, overwrite: bool) -> Self {
        let to_load = engine.resources.request_all_resources(overwrite);

        Self {
            to_load,
            done: false,
        }
    }
}

impl super::Widget for ResourceLoader {
    fn title(&self) -> String {
        String::from("Resource Loader")
    }

    fn resizable(&self) -> bool { false }
    fn closable(&self) -> bool { false }

    fn update_open_status(&self, open: &mut bool) {
        if self.done { *open = false; }
    }

    fn draw_with_ctx(&mut self, _wctx: &mut super::WidgetContext, ctx: &foxtail::Context, ui: &mut egui::Ui, engine: &mut crate::EngineInternals) {
        if engine.resources.requested_resources.len() > 0 {
            let loading = engine.resources.requested_resources.first().unwrap();
            let progress = (engine.resources.requested_resources.len() as f32) / (self.to_load as f32);

            ui.centered_and_justified(|ui| {
                ui.vertical(|ui| {
                    ui.add(egui::ProgressBar::new(progress));
                    ui.label(&format!("Loading \"{}\"...", loading.display()));
                });
            });

            engine.resources.load_next_resource(ctx, &mut engine.world);
        } else {
            self.done = true;
        }
    }
}
