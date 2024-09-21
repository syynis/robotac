use crate::{app::Message, popup::Popup};
use ratatui::{
    crossterm::event::{Event, KeyCode},
    prelude::*,
};

pub struct SeedInput {
    input: String,
}

impl Default for SeedInput {
    fn default() -> Self {
        Self::new()
    }
}

impl SeedInput {
    pub fn new() -> Self {
        Self {
            input: String::new(),
        }
    }

    pub fn update(&mut self, event: &Event) -> Option<Message> {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char(c) => {
                    if c.is_ascii() {
                        self.input.push(c)
                    }
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Enter => {
                    return Some(Message::Reset(Some(
                        self.input
                            .parse::<u64>()
                            .expect("String can only contain digits"),
                    )));
                }
                _ => {}
            }
        }
        None
    }

    pub fn draw(&self) -> impl Widget + '_ {
        Popup::default()
            .title("Input seed for new game".to_string())
            .content(self.input.clone())
    }
}
