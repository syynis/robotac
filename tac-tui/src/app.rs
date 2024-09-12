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

use crate::{board::BoardView, moves::MoveList};

enum Mode {
    Board,
    Moves,
}

pub enum Message {
    Continue,
    Quit,
    MakeMove(TacMove),
}

pub struct App {
    board: Board,
    mode: Mode,
    board_view: BoardView,
    move_list: MoveList,
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
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            match self.update()? {
                Message::Continue => {}
                Message::Quit => break,
                Message::MakeMove(mv) => {
                    self.board.play(mv);
                    self.on_state_change();
                }
            }
        }
        Ok(())
    }

    pub fn update(&mut self) -> io::Result<Message> {
        if event::poll(Duration::from_millis(100))? {
            let event = event::read()?;
            let mut pass_down = false;
            if let Event::Key(key_ev) = event {
                match key_ev.code {
                    KeyCode::Char('q') => return Ok(Message::Quit),
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

        Ok(Message::Continue)
    }

    fn on_state_change(&mut self) {
        self.board_view.on_state_change(&self.board);
        self.move_list.on_state_change(&self.board);
    }

    fn draw(&self, frame: &mut Frame) {
        let horizontal =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let vertical = Layout::vertical([Constraint::Percentage(75), Constraint::Percentage(25)]);
        let [board, right] = horizontal.areas(frame.area());
        let [moves, hand] = vertical.areas(right);
        frame.render_widget(self.board_view.draw(), board);
        frame.render_widget(self.move_list.draw(), moves);
    }
}
