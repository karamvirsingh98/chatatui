use ratatui::widgets::{Block, Paragraph, Widget};

#[derive(Clone, Default)]
pub struct InputWidget {
    pub active: bool,
    pub value: String,
}

impl InputWidget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle(&mut self, active: bool) {
        self.active = active
    }

    pub fn push(&mut self, ch: char) {
        self.value.push(ch);
    }

    pub fn pop(&mut self) {
        self.value.pop();
    }

    pub fn reset(&mut self) {
        self.value = String::new();
    }
}

impl Widget for &InputWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let input: Paragraph<'_> = Paragraph::new(self.value.clone()).block(Block::bordered());
        Widget::render(input, area, buf);
    }
}
