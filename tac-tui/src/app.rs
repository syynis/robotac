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

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct BoardPoint {
    x: f64,
    y: f64,
    color: Color,
}

pub struct App {
    board: Board,
    points: [BoardPoint; 64],
    focused_square: u8,
    marker: Marker,
}

fn tac_color_to_term_color(tac_color: tac_types::Color) -> Color {
    match tac_color {
        tac_types::Color::Black => Color::Black,
        tac_types::Color::Blue => Color::Blue,
        tac_types::Color::Green => Color::Green,
        tac_types::Color::Red => Color::Red,
    }
}

impl App {
    pub fn new() -> Self {
        let mut points = [BoardPoint::default(); 64];
        for i in (0..64) {
            let angle = i as f64 / 64.0 * TAU;
            let (x, y) = (angle.cos() * 64.0, angle.sin() * 64.0);
            points[i] = BoardPoint {
                x,
                y,
                color: Color::White,
            }
        }
        Self {
            board: Board::new(),
            points,
            focused_square: 0,
            marker: Marker::HalfBlock,
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
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Right | KeyCode::Char('l') => {
                            self.focused_square = (self.focused_square + 63) % 64;
                        }
                        KeyCode::Left | KeyCode::Char('h') => {
                            self.focused_square = (self.focused_square + 1) % 64;
                        }
                        _ => {}
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

    fn on_tick(&mut self) {
        self.points
            .iter_mut()
            .enumerate()
            .for_each(|(idx, mut point)| {
                // This is a valid casting because `points` has a fixed size of 64
                let idx = idx as u8;
                if let Some(c) = self.board.color_on(Square(idx)) {
                    point.color = tac_color_to_term_color(c);
                }
            });
    }

    fn draw(&self, frame: &mut Frame) {
        let horizontal =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let vertical = Layout::vertical([Constraint::Percentage(75), Constraint::Percentage(25)]);
        let [board, right] = horizontal.areas(frame.area());
        let [moves, hand] = vertical.areas(right);
        frame.render_widget(self.board_canvas(), board);
        frame.render_widget(self.moves_list(), moves);
    }

    fn board_canvas(&self) -> impl Widget + '_ {
        // diameter + padding
        let size = 64.0 + 16.0;
        let bounds = [-size, size];
        Canvas::default()
            .block(Block::bordered().title("Board"))
            .marker(self.marker)
            .paint(move |ctx| {
                ctx.draw(&ColoredPoints {
                    points: &self.points,
                });
                let angle = self.focused_square as f64 / 64.0 * TAU;
                let (x, y) = (angle.cos() * 70.0, angle.sin() * 70.0);
                ctx.draw(&Rectangle {
                    x,
                    y,
                    width: 0.01,
                    height: 0.01,
                    color: Color::Yellow,
                })
            })
            .x_bounds(bounds)
            .y_bounds(bounds)
    }

    fn moves_list(&self) -> impl Widget + '_ {
        let block = Block::new()
            .title(Line::raw("Moves").left_aligned())
            .borders(Borders::TOP);
        let moves = self.board.get_moves(self.board.current_player());
        let items = moves.iter().map(|e| format!("{}", e)).into_iter();
        List::new(items).block(block).highlight_symbol(">")
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
struct ColoredPoints<'a> {
    pub points: &'a [BoardPoint],
}

impl<'a> Shape for ColoredPoints<'a> {
    fn draw(&self, painter: &mut ratatui::widgets::canvas::Painter) {
        for BoardPoint { x, y, color } in self.points {
            if let Some((x, y)) = painter.get_point(*x, *y) {
                painter.paint(x, y, *color);
            }
        }
    }
}
