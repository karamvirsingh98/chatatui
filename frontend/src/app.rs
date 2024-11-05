use std::{
    io,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crossterm::event::{Event, EventStream, KeyCode, KeyEventKind};
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use ratatui::{
    layout::{Constraint, Layout, Position, Rect},
    prelude::Backend,
    widgets::{Block, Paragraph},
    Frame, Terminal,
};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};

use crate::widgets::{centered_rect, chat::ChatWidget, input::InputWidget};

#[derive(PartialEq)]
pub enum AppMode {
    Login,
    Chat,
    Exit,
}

pub struct App {
    // general
    pub mode: AppMode,
    pub name: String,

    // widgets
    pub input: InputWidget,
    pub chat: ChatWidget,

    // net
    pub ws_sender: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>,
}

impl App {
    const FRAMES_PER_SECOND: f32 = 60.0;

    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let (socket, _) = connect_async("ws://localhost:3000/ws").await?;
        let (ws_sender, mut ws_receiver) = socket.split();

        let chat = ChatWidget::new();

        let messages = chat.messages.clone();
        tokio::spawn(async move {
            while let Some(Ok(Message::Text(msg))) = ws_receiver.next().await {
                if let Ok(mut messages) = messages.write() {
                    if let Ok(message) = serde_json::from_str(&msg) {
                        messages.push(message);
                    };
                }
            }
        });

        Ok(Self {
            mode: AppMode::Login,
            name: String::new(),
            input: InputWidget::new(),
            chat,
            ws_sender,
        })
    }

    pub async fn start<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> io::Result<bool> {
        let mut events = EventStream::new();

        let period = Duration::from_secs_f32(1.0 / Self::FRAMES_PER_SECOND);
        let mut interval = tokio::time::interval(period);

        while self.mode != AppMode::Exit {
            tokio::select! {
                _ = interval.tick() => { terminal.draw(|frame| self.draw(frame))?; },
                Some(Ok(event)) = events.next() => self.handle_event(&event).await,
            }
        }

        Ok(true)
    }

    async fn handle_event(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            if key.kind == KeyEventKind::Press {
                match self.input.active {
                    true => match key.code {
                        KeyCode::Enter => match self.mode {
                            AppMode::Login => self.login(),
                            AppMode::Chat => self.send_message().await,
                            _ => {}
                        },
                        KeyCode::Char(key) => self.input.push(key),
                        KeyCode::Backspace => self.input.pop(),
                        _ => {}
                    },
                    false => match key.code {
                        KeyCode::Enter => self.input.toggle(true),
                        KeyCode::Char('q') => self.mode = AppMode::Exit,
                        _ => {}
                    },
                }
            }
        }
    }

    async fn send_message(&mut self) {
        let message = chatatui_types::Message {
            ts: get_timestamp(),
            sender: self.name.clone(),
            text: self.input.value.clone(),
        };

        if message.text.len() > 0 {
            if let Ok(message) = serde_json::to_string(&message) {
                let res = self.ws_sender.send(Message::Text(message)).await;
                if let Err(e) = res {
                    eprintln!("{e:#?}")
                }
            }
        }

        self.input.reset();
        self.input.toggle(false);
    }

    fn draw(&mut self, frame: &mut Frame) {
        let [header_area, chat_area, input_area] = Layout::vertical([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(3),
        ])
        .areas(frame.area());

        let [title, user] =
            Layout::horizontal([Constraint::Fill(1), Constraint::Max(10)]).areas(header_area);

        // header widget
        let header = Paragraph::new("chatatui!").block(Block::bordered());
        let name = Paragraph::new(self.name.clone()).block(Block::bordered());

        frame.render_widget(header, title);
        frame.render_widget(name, user);

        if self.mode == AppMode::Login {
            self.input.toggle(true);

            let widget = centered_rect(Constraint::Max(5), Constraint::Percentage(50), chat_area);

            let block = Block::bordered().title("Login");

            update_cursor(
                frame,
                block.inner(widget),
                self.input.value.len() as u16 + 1,
            );

            frame.render_widget(&self.input, block.inner(widget));
            frame.render_widget(block, widget);
        } else {
            if self.input.active {
                frame.set_cursor_position(Position::new(
                    input_area.x + self.input.value.len() as u16 + 1,
                    input_area.y + 1,
                ))
            };

            frame.render_widget(&self.chat, chat_area);
            frame.render_widget(&self.input, input_area);
            if self.input.active {
                update_cursor(frame, input_area, self.input.value.len() as u16 + 1);
            }
        }
    }

    fn login(&mut self) {
        self.name = self.input.value.clone();

        self.input.toggle(false);
        self.input.reset();

        self.mode = AppMode::Chat;
    }
}

pub fn get_timestamp() -> i64 {
    let start = SystemTime::now();
    let time = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    time.as_millis() as i64
}

pub fn update_cursor(frame: &mut Frame, area: Rect, pos: u16) {
    frame.set_cursor_position(Position::new(area.x + pos, area.y + 1))
}
