use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

#[derive(Debug, Default)]
pub struct Popup<'a> {
    title: Line<'a>,
    content: Text<'a>,
    border_style: Style,
    title_style: Style,
    style: Style,
}

impl<'a> Popup<'a> {
    pub fn title(self, title: String) -> Self {
        Self {
            title: Line::from(title),
            ..self
        }
    }
    pub fn content(self, content: String) -> Self {
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
