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
    Quit,
    StateChange,
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
        let tick_rate = Duration::from_millis(8);
        let mut last_tick = Instant::now();
        let mut need_update = false;
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if event::poll(timeout)? {
                let event = event::read()?;
                let mut pass_down = false;
                if let Event::Key(key_ev) = event {
                    match key_ev.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Char('h') => self.mode = Mode::Board,
                        KeyCode::Char('l') => self.mode = Mode::Moves,
                        _ => {
                            pass_down = true;
                        }
                    }
                }
                if pass_down {
                    match self.mode {
                        Mode::Board => self.board_view.update(&event),
                        Mode::Moves => self.move_list.update(&event),
                    }
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if need_update {
                    self.on_tick();
                }
                last_tick = Instant::now();
            }
        }
        Ok(())
    }

    pub fn update(&mut self) -> io::Result<Message> {}

    fn on_state_change(&mut self) {
        match self.mode {
            Mode::Board => self.board_view.on_change(&self.board),
            Mode::Moves => self.move_list.on_change(&self.board),
        }
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
