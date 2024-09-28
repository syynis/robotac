use std::{
    fs::File,
    io::{self, Write},
    time::Duration,
};

use mcts::{manager::Manager, policies::UCTPolicy};
use ratatui::{
    crossterm::event::{self, Event, KeyCode},
    layout::{Constraint, Layout, Rect},
    DefaultTerminal, Frame,
};
use robotac::{board::Board, history::History, TacAI, TacEval};
use tac_types::TacMove;

use crate::{
    ai_debug::AiDebugView,
    board::BoardView,
    debug::DebugView,
    history::{LoadHistory, SaveHistory},
    moves::MoveList,
    seed_input::SeedInput,
};

enum Mode {
    Moves,
    SeedEdit,
    SaveHistory,
    LoadHistory,
}

impl Mode {
    fn need_input(&self) -> bool {
        match self {
            Mode::Moves => false,
            Mode::SeedEdit => true,
            Mode::SaveHistory => true,
            Mode::LoadHistory => false,
        }
    }
}

pub enum Message {
    Quit,
    MakeMove(TacMove),
    Reset(Option<u64>),
    SaveHistory(String),
    LoadHistory(String),
}

pub struct App {
    board: Board,
    history: History,
    mode: Mode,
    ai: Manager<TacAI>,
    board_view: BoardView,
    move_list: MoveList,
    debug: DebugView,
    ai_debug: AiDebugView,
    seed_input: SeedInput,
    save_history: SaveHistory,
    load_history: LoadHistory,
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
        let ai = Manager::new(board.clone(), TacAI, UCTPolicy(0.7), TacEval);
        let move_list = MoveList::new(&board);
        Self {
            board,
            history: History::new(0),
            mode: Mode::Moves,
            ai,
            board_view: BoardView::default(),
            move_list,
            debug: DebugView,
            ai_debug: AiDebugView,
            seed_input: SeedInput::default(),
            save_history: SaveHistory::default(),
            load_history: LoadHistory::default(),
            previous_seed,
        }
    }

    pub fn new_board(&mut self, seed: u64) {
        self.board = Board::new_with_seed(seed);
        self.history = History::new(seed);
        self.on_state_change();
    }

    pub fn load_history(&mut self, history: &History) {
        self.board = history.board_with_history();
        self.history = history.clone();
        self.on_state_change();
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> io::Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;
            if let Some(message) = self.update() {
                match message {
                    Message::Quit => break,
                    Message::MakeMove(mv) => {
                        self.board.play(&mv);
                        self.history.moves.push(mv);
                        self.on_state_change();
                    }
                    Message::Reset(seed) => {
                        let seed = seed.unwrap_or(self.previous_seed);
                        self.new_board(seed);
                        self.mode = Mode::Moves;
                        self.previous_seed = seed;
                    }
                    Message::SaveHistory(s) => {
                        let _ = Self::write_history_to_file(&self.history, &s);
                        self.mode = Mode::Moves
                    }
                    Message::LoadHistory(s) => {
                        self.mode = Mode::Moves;
                        if let Ok(content) = std::fs::read_to_string(format!("histories/{}", s)) {
                            if let Ok(history) = ron::de::from_str::<History>(&content) {
                                self.load_history(&history);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn write_history_to_file(history: &History, name: &str) -> std::io::Result<()> {
        let mut file = File::create(format!("histories/{}.hist", name))?;
        let ron = ron::ser::to_string_pretty(history, ron::ser::PrettyConfig::default()).unwrap();
        let _ = file.write_all(&ron.into_bytes());
        Ok(())
    }

    pub fn update(&mut self) -> Option<Message> {
        if event::poll(Duration::from_millis(10)).ok()? {
            let event = event::read().ok()?;
            let mut pass_down = false;
            if let Event::Key(key_ev) = event {
                if matches!(key_ev.code, KeyCode::Esc) {
                    self.mode = Mode::Moves;
                    return None;
                }

                if !self.mode.need_input() {
                    match key_ev.code {
                        KeyCode::Char('q') => return Some(Message::Quit),
                        KeyCode::Char('m') => self.mode = Mode::Moves,
                        KeyCode::Char('n') => self.mode = Mode::SeedEdit,
                        KeyCode::Char('r') => return Some(Message::Reset(None)),
                        KeyCode::Char('s') => self.mode = Mode::SaveHistory,
                        KeyCode::Char('l') => self.mode = Mode::LoadHistory,
                        KeyCode::Char('p') => self.ai.playout_n(1000),
                        _ => {
                            pass_down = true;
                        }
                    }
                } else {
                    pass_down = true;
                }
            }
            if pass_down {
                return match self.mode {
                    Mode::Moves => self.move_list.update(&event),
                    Mode::SeedEdit => self.seed_input.update(&event),
                    Mode::SaveHistory => self.save_history.update(&event),
                    Mode::LoadHistory => self.load_history.update(&event),
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
        let vertical = Layout::vertical([Constraint::Percentage(40), Constraint::Percentage(60)]);
        let [board, right] = horizontal.areas(frame.area());
        let [moves, debug] = vertical.areas(right);
        frame.render_widget(self.board_view.draw(), board);
        frame.render_widget(self.move_list.draw(), moves);
        frame.render_widget(self.debug.draw(&self.board), debug);
        // frame.render_widget(self.ai_debug.draw(&self.ai), debug);
        match self.mode {
            Mode::SeedEdit => {
                let area = Rect {
                    x: frame.area().width / 2 - 10,
                    y: frame.area().height / 2 - 1,
                    width: 30,
                    height: 3,
                };
                frame.render_widget(self.seed_input.draw(), area);
            }
            Mode::SaveHistory => {
                let area = Rect {
                    x: frame.area().width / 2 - 10,
                    y: frame.area().height / 2 - 1,
                    width: 30,
                    height: 3,
                };
                frame.render_widget(self.save_history.draw(), area);
            }
            Mode::LoadHistory => {
                let area = Rect {
                    x: frame.area().width / 2 - frame.area().width / 8,
                    y: frame.area().height / 4,
                    width: frame.area().width / 4,
                    height: frame.area().height / 2,
                };
                frame.render_widget(self.load_history.draw(), area);
            }
            _ => {}
        }
        if matches!(self.mode, Mode::SeedEdit) {}
    }
}
