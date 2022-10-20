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
    input: String,
    input_history: VecDeque<String>,
    input_history_cursor: usize,
}

impl Console {
    pub fn new() -> Self {
        Self {
            buf: ConsoleBuf::new(),
            input: String::new(),
            input_history: VecDeque::new(),
            input_history_cursor: 0,
        }
    }

    pub fn draw(&mut self, egui_ctx: &egui::Context) -> Option<Command> {
        let mut res = None;
        egui::Window::new("console")
            .resizable(true)
            .show(egui_ctx, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.buf)
                    .code_editor()
                    .desired_rows(32)
                    .desired_width(std::f32::INFINITY)
                    .interactive(false)
                );
                let response = ui.add(
                    egui::TextEdit::singleline(&mut self.input)
                    .code_editor()
                    .desired_width(std::f32::INFINITY)
                    .hint_text("Enter command...")
                );
                if response.has_focus() {
                    if ui.input().key_pressed(egui::Key::ArrowUp) {
                        self.input_history_cursor += 1;
                        if self.input_history_cursor > self.input_history.len() {
                            self.input_history_cursor -= 1;
                        }
                        self.input = self.input_history[self.input_history_cursor-1].clone();
                    } else if ui.input().key_pressed(egui::Key::ArrowDown) {
                        if self.input_history_cursor > 1 {
                            self.input_history_cursor -= 1;
                            self.input = self.input_history[self.input_history_cursor-1].clone();
                        }
                    }
                }
                if response.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                    if self.input.len() > 0 {
                        self.buf.push(format!(">>> {}", self.input));
                        self.input_history.push_front(self.input.clone());
                        if self.input_history.len() > 32 { self.input_history.pop_back(); }
                        self.input_history_cursor = 0;
                        res = Some(Command::new(self.input.clone()));
                        self.input = String::new();
                    }
                    response.request_focus();
                }
            });
        res
    }
}

pub struct Command {
    pub name: String,
    pub args: Vec<String>,
}

impl Command {
    pub fn new(cmd: String) -> Self {
        let mut split = cmd.split(" ");
        let name = split.next().expect("Empty command? How did we get here?");
        let args: Vec<String> = split.map(|s| s.to_string()).collect();
        Self {
            name: name.to_string(),
            args: args,
        }
    }
}
