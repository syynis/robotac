use std::{io, time::Duration};

use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Layout, Rect},
    DefaultTerminal, Frame,
};
use robotac::board::Board;
use tac_types::TacMove;

use crate::{board::BoardView, debug::DebugView, moves::MoveList, seed_input::SeedInput};

enum Mode {
    Moves,
    SeedEdit,
}

pub enum Message {
    Quit,
    MakeMove(TacMove),
    Reset(Option<u64>),
}

pub struct App {
    board: Board,
    mode: Mode,
    board_view: BoardView,
    move_list: MoveList,
    debug: DebugView,
    seed_input: SeedInput,
    previous_seed: u64,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let previous_seed = 0;
        let board = Board::new_with_seed(previous_seed);
        let move_list = MoveList::new(&board);
        Self {
            board,
            mode: Mode::Moves,
            board_view: BoardView::default(),
            move_list,
            debug: DebugView,
            seed_input: SeedInput::default(),
            previous_seed,
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
                    Message::Reset(seed) => {
                        let seed = seed.unwrap_or(self.previous_seed);
                        self.new_board(seed);
                        self.on_state_change();
                        self.mode = Mode::Moves;
                        self.previous_seed = seed;
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
                    KeyCode::Char('m') => self.mode = Mode::Moves,
                    KeyCode::Char('n') => self.mode = Mode::SeedEdit,
                    KeyCode::Char('r') => return Some(Message::Reset(None)),
                    _ => {
                        pass_down = true;
                    }
                }
            }
            if pass_down {
                return match self.mode {
                    Mode::Moves => self.move_list.update(&event),
                    Mode::SeedEdit => self.seed_input.update(&event),
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
        if matches!(self.mode, Mode::SeedEdit) {
            let area = Rect {
                x: frame.area().width / 2 - 10,
                y: frame.area().height / 2 - 1,
                width: 30,
                height: 3,
            };
            frame.render_widget(self.seed_input.draw(), area);
        }
    }
}
