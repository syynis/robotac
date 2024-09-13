use ratatui::{
    crossterm::event::Event,
    widgets::{Block, Paragraph, Widget},
};
use robotac::board::Board;

use crate::app::Message;

pub struct DebugView;

impl DebugView {
    pub fn update(&mut self, _event: &Event) -> Option<Message> {
        None
    }

    pub fn draw(&self, board: &Board) -> impl Widget + '_ {
        Paragraph::new(format!("{:?}", board)).block(Block::bordered().title("Debug state"))
    }
}
