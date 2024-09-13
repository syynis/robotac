use std::f64::consts::TAU;

use ratatui::{
    crossterm::event::Event,
    style::Color,
    symbols::Marker,
    widgets::{
        canvas::{Canvas, Rectangle, Shape},
        Block, Widget,
    },
};
use tac_types::{Home, Square, ALL_COLORS};

use crate::app::Message;

const CANVAS_SIZE: f64 = 256.0;
const CANVAS_PADDING: f64 = 32.0;

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
    outside: [u8; 4],
    homes: [Home; 4],
}

impl Default for BoardView {
    fn default() -> Self {
        Self::new()
    }
}

impl BoardView {
    pub fn new() -> Self {
        let mut points = [BoardPoint::default(); 64];
        (0..64).for_each(|i| {
            let angle = i as f64 / 64.0 * TAU;
            let (x, y) = (angle.cos() * CANVAS_SIZE, angle.sin() * CANVAS_SIZE);
            points[i] = BoardPoint {
                x,
                y,
                color: Color::Rgb(255, 255, 255),
            }
        });
        Self {
            points,
            outside: [4; 4],
            homes: [Home::default(); 4],
        }
    }

    pub fn update(&mut self, _event: &Event) -> Option<Message> {
        None
    }
    pub fn on_state_change(&mut self, board: &robotac::board::Board) {
        for (idx, p) in self.points.iter_mut().enumerate() {
            // This is a valid casting because `points` has a fixed size of 64
            let idx = idx as u8;
            if let Some(c) = board.color_on(Square(idx)) {
                p.color = term_color(c);
            } else {
                p.color = Color::Rgb(255, 255, 255);
            }
        }
        for (idx, c) in ALL_COLORS.iter().enumerate() {
            self.outside[idx] = board.num_outside(*c);
            self.homes[idx] = board.home(*c);
        }
    }

    pub fn draw(&self) -> impl Widget + '_ {
        // diameter + padding
        let size = CANVAS_SIZE + CANVAS_PADDING;
        let bounds = [-size, size];
        Canvas::default()
            .block(Block::bordered().title("Board"))
            .marker(Marker::Bar)
            .paint(move |ctx| {
                ctx.draw(&ColoredPoints {
                    points: &self.points,
                });

                let resolution = 4;
                for i in 0..64 / resolution {
                    let angle = (i * resolution) as f64 / 64.0 * TAU;
                    let (x, y) = (
                        angle.cos() * (CANVAS_SIZE + 16.0),
                        angle.sin() * (CANVAS_SIZE + 16.0),
                    );
                    if i % resolution == 0 {
                        ctx.draw(&Rectangle {
                            x,
                            y,
                            width: 0.01,
                            height: 0.01,
                            color: term_color(ALL_COLORS[i / resolution]),
                        });
                    }
                    ctx.print(
                        x,
                        y + if i % resolution == 0 {
                            8.0 * y.signum()
                        } else {
                            0.0
                        },
                        format!("{}", i * resolution),
                    );
                }

                for (idx, home) in self.homes.iter().enumerate() {
                    let angle = (idx * 16) as f64 / 64.0 * TAU;
                    for p in 1..=4 {
                        let (x, y) = (
                            angle.cos() * (CANVAS_SIZE - 32.0 * p as f64),
                            angle.sin() * (CANVAS_SIZE - 32.0 * p as f64),
                        );
                        ctx.draw(&Rectangle {
                            x,
                            y,
                            width: 0.01,
                            height: 0.01,
                            color: if home.is_free(p - 1) {
                                Color::Rgb(255, 255, 255)
                            } else {
                                term_color(ALL_COLORS[idx])
                            },
                        });
                    }
                }
                let dist = CANVAS_SIZE;
                let idx_pos = [
                    (dist - CANVAS_PADDING, -dist),
                    (dist - CANVAS_PADDING, dist),
                    (-dist, dist),
                    (-dist, -dist),
                ];
                for (idx, amount) in self.outside.iter().enumerate() {
                    let (start_x, start_y) = idx_pos[idx];
                    for i in 0..*amount {
                        ctx.draw(&Rectangle {
                            x: start_x + (i * CANVAS_PADDING as u8 / 2) as f64,
                            y: start_y,
                            width: 0.1,
                            height: 0.1,
                            color: term_color(ALL_COLORS[idx]),
                        });
                    }
                }
            })
            .x_bounds(bounds)
            .y_bounds(bounds)
    }
}

fn term_color(tac_color: tac_types::Color) -> Color {
    match tac_color {
        tac_types::Color::Black => Color::Black,
        tac_types::Color::Blue => Color::Blue,
        tac_types::Color::Green => Color::Green,
        tac_types::Color::Red => Color::Red,
    }
}
