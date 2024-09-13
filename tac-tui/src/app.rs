use std::{
    f64::consts::TAU,
    io,
    time::{Duration, Instant},
};

use itertools::Itertools;
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Layout, Rect},
    style::{palette::tailwind::SLATE, Color, Modifier, Style, Stylize},
    symbols::Marker,
    text::{Line, Text},
    widgets::{
        canvas::{Canvas, Circle, Map, MapResolution, Points, Rectangle, Shape},
        Block, Borders, List, ListItem, Widget,
    },
    DefaultTerminal, Frame,
};
use robotac::board::Board;
use tac_types::{Square, TacMove};

use crate::{board::BoardView, debug::DebugView, moves::MoveList};

enum Mode {
    Board,
    Moves,
}

pub enum Message {
    Quit,
    MakeMove(TacMove),
}

pub struct App {
    board: Board,
    mode: Mode,
    board_view: BoardView,
    move_list: MoveList,
    debug: DebugView,
}

impl App {
    pub fn new() -> Self {
        let board = Board::new();
        let move_list = MoveList::new(&board);
        Self {
            board,
            mode: Mode::Board,
            board_view: BoardView::new(),
            move_list,
            debug: DebugView,
        }
    }

    pub fn new_board(&mut self, seed: u64) {
        self.board = Board::new_with_seed(seed);
        self.on_state_change();
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            if let Some(message) = self.update() {
                match message {
                    Message::Quit => break,
                    Message::MakeMove(mv) => {
                        self.board.play(mv);
                        self.on_state_change();
                    }
                }
            }
        }
        Ok(())
    }

    pub fn update(&mut self) -> Option<Message> {
        if event::poll(Duration::from_millis(100)).ok()? {
            let event = event::read().ok()?;
            let mut pass_down = false;
            if let Event::Key(key_ev) = event {
                match key_ev.code {
                    KeyCode::Char('q') => return Some(Message::Quit),
                    KeyCode::Char('h') => self.mode = Mode::Board,
                    KeyCode::Char('l') => self.mode = Mode::Moves,
                    _ => {
                        pass_down = true;
                    }
                }
            }
            if pass_down {
                return match self.mode {
                    Mode::Board => self.board_view.update(&event),
                    Mode::Moves => self.move_list.update(&event),
                };
            }
        }
        None
    }

    fn on_state_change(&mut self) {
        self.board_view.on_state_change(&self.board);
        self.move_list.on_state_change(&self.board);
    }

    fn draw(&self, frame: &mut Frame) {
        let horizontal =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let vertical = Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)]);
        let [board, right] = horizontal.areas(frame.area());
        let [moves, debug] = vertical.areas(right);
        frame.render_widget(self.board_view.draw(), board);
        frame.render_widget(self.move_list.draw(), moves);
        frame.render_widget(self.debug.draw(&self.board), debug);
    }
}
