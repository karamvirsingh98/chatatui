use std::sync::{Arc, RwLock};

use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, List, Widget},
};

#[derive(Default)]
pub struct ChatWidget {
    pub messages: Arc<RwLock<Vec<chatatui_types::Message>>>,
}

impl ChatWidget {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, message: chatatui_types::Message) {
        if let Ok(mut messages) = self.messages.write() {
            messages.push(message)
        }
    }
}

impl Widget for &ChatWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer) {
        let messages = self.messages.read().unwrap().clone();
        let chat = List::new(messages.into_iter().map(|message| {
            // message
            Line::from(vec![
                Span::styled(message.ts.to_string(), Style::new().fg(Color::Blue)),
                Span::from(" "),
                Span::styled(message.sender, Style::new().fg(Color::Yellow)),
                Span::from(" "),
                Span::styled(message.text, Style::new()),
            ])
        }))
        .block(Block::bordered());
        Widget::render(chat, area, buf);
    }
}
