use mcts::manager::Manager;
use ratatui::{
    crossterm::event::Event,
    widgets::{Block, Paragraph, Widget},
};
use robotac::TacAI;

use crate::app::Message;

pub struct AiDebugView;

impl AiDebugView {
    pub fn update(&mut self, _event: &Event) -> Option<Message> {
        None
    }

    pub fn draw(&self, ai: &Manager<TacAI>) -> impl Widget + '_ {
        let mut string = String::new();
        for s in ai.stats() {
            string.push_str(&format!("{:?}\n", s));
        }
        Paragraph::new(string).block(Block::bordered().title("Debug state"))
    }
}
