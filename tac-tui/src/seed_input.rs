use crate::app::Message;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{Event, KeyCode},
    layout::Rect,
    style::Style,
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

pub struct SeedInput {
    input: String,
}

impl SeedInput {
    pub fn new() -> Self {
        Self {
            input: String::new(),
        }
    }

    pub fn update(&mut self, event: &Event) -> Option<Message> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char(c) => {
                    if c.is_digit(10) {
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
            },
            _ => {}
        }
        None
    }

    pub fn draw(&self) -> impl Widget + '_ {
        Popup::default()
            .title("Input seed for new game".to_string())
            .content(self.input.clone())
    }
}

#[derive(Debug, Default)]
struct Popup<'a> {
    title: Line<'a>,
    content: Text<'a>,
    border_style: Style,
    title_style: Style,
    style: Style,
}

impl<'a> Popup<'a> {
    fn title(self, title: String) -> Self {
        Self {
            title: Line::from(title),
            ..self
        }
    }
    fn content(self, content: String) -> Self {
        Self {
            content: Text::from(content),
            ..self
        }
    }
}

impl Widget for Popup<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // ensure that all cells under the popup are cleared to avoid leaking content
        Clear.render(area, buf);
        let block = Block::new()
            .title(self.title)
            .title_style(self.title_style)
            .borders(Borders::ALL)
            .border_style(self.border_style);
        Paragraph::new(self.content)
            .wrap(Wrap { trim: true })
            .style(self.style)
            .left_aligned()
            .block(block)
            .render(area, buf);
    }
}
