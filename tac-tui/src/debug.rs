use ratatui::{
    crossterm::event::{Event, KeyCode},
    text::Line,
    widgets::{Block, Borders, List, Paragraph, StatefulWidget, Widget},
};
use robotac::board::Board;
use tac_types::{Card, TacMove};

use crate::app::Message;

pub struct DebugView;

impl DebugView {
    pub fn update(&mut self, _event: &Event) -> Option<Message> {
        None
    }

    pub fn draw(&self, board: &Board) -> impl Widget + '_ {
        Paragraph::new(format!("{:?}", board))
    }
}
