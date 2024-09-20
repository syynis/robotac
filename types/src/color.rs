use crate::Square;

pub const ALL_COLORS: [Color; 4] = [Color::Black, Color::Blue, Color::Green, Color::Red];
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Color {
    Black,
    Blue,
    Green,
    Red,
}

impl Color {
    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Color::Black => Color::Blue,
            Color::Blue => Color::Green,
            Color::Green => Color::Red,
            Color::Red => Color::Black,
        }
    }

    #[must_use]
    pub const fn prev(self) -> Self {
        match self {
            Color::Black => Color::Red,
            Color::Blue => Color::Black,
            Color::Green => Color::Blue,
            Color::Red => Color::Green,
        }
    }

    #[must_use]
    pub const fn partner(self) -> Self {
        self.next().next()
    }

    #[must_use]
    pub const fn home(self) -> Square {
        match self {
            Color::Black => Square(0),
            Color::Blue => Square(16),
            Color::Green => Square(32),
            Color::Red => Square(48),
        }
    }
}
