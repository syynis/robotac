use serde::{Deserialize, Serialize};

use crate::Square;

pub const ALL_COLORS: [Color; 4] = [Color::Black, Color::Blue, Color::Green, Color::Red];
#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Color {
    Black,
    Blue,
    Green,
    Red,
}

impl From<Color> for usize {
    fn from(value: Color) -> Self {
        value as usize
    }
}

impl From<usize> for Color {
    fn from(value: usize) -> Self {
        unsafe { std::mem::transmute(value as u8) }
    }
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

    #[must_use]
    /// How many players are between self and other
    pub fn between(self, other: Self) -> usize {
        let mut res = 0;
        let mut curr = self.next();
        while curr != other {
            curr = curr.next();
            res += 1;
        }
        res
    }
}
