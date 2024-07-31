use crate::Square;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Color {
    Black,
    Blue,
    Green,
    Red,
}

impl Color {
    pub const fn next(self) -> Self {
        match self {
            Color::Black => Color::Blue,
            Color::Blue => Color::Green,
            Color::Green => Color::Red,
            Color::Red => Color::Black,
        }
    }
    pub const fn prev(self) -> Self {
        match self {
            Color::Black => Color::Red,
            Color::Blue => Color::Black,
            Color::Green => Color::Blue,
            Color::Red => Color::Green,
        }
    }

    pub const fn partner(self) -> Self {
        match self {
            Color::Black => Color::Green,
            Color::Blue => Color::Red,
            Color::Green => Color::Black,
            Color::Red => Color::Blue,
        }
    }

    pub const fn home(self) -> Square {
        match self {
            Color::Black => Square(0),
            Color::Blue => Square(16),
            Color::Green => Square(32),
            Color::Red => Square(48),
        }
    }
}
