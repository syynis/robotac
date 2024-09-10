use std::f64::consts::TAU;

use ratatui::{
    crossterm::event::{Event, KeyCode},
    style::Color,
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Rectangle, Shape},
        Block, Widget,
    },
};
use tac_types::Square;

use crate::app::App;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
struct BoardPoint {
    x: f64,
    y: f64,
    color: Color,
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

pub struct BoardView {
    points: [BoardPoint; 64],
    focused_square: u8,
}

impl BoardView {
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
            points,
            focused_square: 0,
        }
    }

    pub fn update(&mut self, event: &Event) {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Right | KeyCode::Char('j') => {
                    self.focused_square = (self.focused_square + 63) % 64;
                }
                KeyCode::Left | KeyCode::Char('k') => {
                    self.focused_square = (self.focused_square + 1) % 64;
                }
                _ => {}
            },
            _ => {}
        }
    }
    pub fn on_change(&mut self, board: &robotac::board::Board) {
        self.points
            .iter_mut()
            .enumerate()
            .for_each(|(idx, mut point)| {
                // This is a valid casting because `points` has a fixed size of 64
                let idx = idx as u8;
                if let Some(c) = board.color_on(Square(idx)) {
                    point.color = tac_color_to_term_color(c);
                }
            });
    }

    pub fn draw(&self) -> impl Widget + '_ {
        // diameter + padding
        let size = 64.0 + 16.0;
        let bounds = [-size, size];
        Canvas::default()
            .block(Block::bordered().title("Board"))
            .marker(Marker::HalfBlock)
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
}

fn tac_color_to_term_color(tac_color: tac_types::Color) -> Color {
    match tac_color {
        tac_types::Color::Black => Color::Black,
        tac_types::Color::Blue => Color::Blue,
        tac_types::Color::Green => Color::Green,
        tac_types::Color::Red => Color::Red,
    }
}
