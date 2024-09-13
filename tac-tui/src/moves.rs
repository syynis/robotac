use std::io;

use ratatui::{
    crossterm::event::{Event, KeyCode},
    text::Line,
    widgets::{Block, Borders, List, StatefulWidget, Widget},
};
use tac_types::{Card, TacMove};

use crate::app::Message;

pub struct MoveList {
    moves: Vec<TacMove>,
    selected: usize,
}

impl MoveList {
    pub fn new(board: &robotac::board::Board) -> Self {
        Self {
            moves: board.get_moves(board.current_player()),
            selected: 0,
        }
    }
    pub fn update(&mut self, event: &Event) -> Option<Message> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Right | KeyCode::Char('j') => {
                    self.selected = (self.selected + 1).min(self.moves.len() - 1);
                }
                KeyCode::Left | KeyCode::Char('k') => {
                    self.selected = self.selected.saturating_sub(1);
                }
                KeyCode::Enter => {
                    let mv = self.moves[self.selected].clone();
                    return Some(Message::MakeMove(mv));
                }
                _ => {}
            },
            _ => {}
        }
        None
    }
    pub fn on_state_change(&mut self, board: &robotac::board::Board) {
        *self = MoveList::new(board);
    }

    pub fn draw(&self) -> impl Widget + '_ {
        let block = Block::new()
            .borders(Borders::ALL)
            .title(Line::raw("Moves").left_aligned());
        let items = self
            .moves
            .iter()
            .enumerate()
            .map(|(idx, e)| format!("{}{}", if idx == self.selected { '>' } else { ' ' }, e))
            .into_iter();
        List::new(items).block(block).highlight_symbol(">")
    }
}