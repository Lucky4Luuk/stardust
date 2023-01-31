use std::collections::VecDeque;

pub struct ConsoleBuf {
    ring: VecDeque<String>,
    max_size: usize,
    string: String,
}

impl ConsoleBuf {
    pub fn new() -> Self {
        Self {
            ring: VecDeque::new(),
            max_size: 32,
            string: String::new(),
        }
    }

    fn rebuild_internal_string(&mut self) {
        self.string = self.ring.iter().map(|s| s.clone()).collect::<Vec<String>>().join("\n");
    }

    pub fn push<S: Into<String>>(&mut self, text: S) {
        self.ring.push_back(text.into());
        if self.ring.len() > self.max_size {
            self.ring.pop_front();
        }
        self.rebuild_internal_string();
    }
}

impl egui::TextBuffer for ConsoleBuf {
    fn is_mutable(&self) -> bool { true }
    fn as_str(&self) -> &str {
        &self.string
    }
    fn insert_text(&mut self, _: &str, _: usize) -> usize { todo!() }
    fn delete_char_range(&mut self, _: std::ops::Range<usize>) { todo!() }
}

pub struct Console {
    pub buf: ConsoleBuf,
}

impl Console {
    pub fn new() -> Self {
        Self {
            buf: ConsoleBuf::new(),
        }
    }
}

impl super::Widget for Console {
    fn title(&self) -> String {
        String::from("Console")
    }

    fn draw(&mut self, ctx: &mut super::WidgetContext, ui: &mut egui::Ui, engine: &mut crate::EngineInternals) {
        // Pull all pending writes from the engine
        while let Some(s) = engine.console_pending_writes.pop_front() {
            self.buf.push(s);
        }

        ui.add(
            egui::TextEdit::multiline(&mut self.buf)
            .code_editor()
            .desired_rows(32)
            .desired_width(std::f32::INFINITY)
            .interactive(false)
        );
    }
}
