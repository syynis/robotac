use serde::{Deserialize, Serialize};

use crate::{bitboard::BitBoard, color::Color};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash, Serialize, Deserialize,
)]
pub struct Square(pub u8);

impl From<u8> for Square {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl Square {
    const MIN: Square = Square(0);
    const MAX: Square = Square(63);

    #[must_use]
    pub const fn bitboard(self) -> BitBoard {
        BitBoard(1 << self.0)
    }

    // Wrap square around range 0 to 63.
    // NOTE This only works if square value is 127 at most
    #[must_use]
    pub const fn make_valid(self) -> Self {
        Self(self.0 & 63)
    }

    #[must_use]
    pub const fn add(self, amount: u8) -> Self {
        Self(self.0 + amount).make_valid()
    }

    #[must_use]
    pub const fn sub(self, amount: u8) -> Self {
        Self(self.0 + 64 - amount).make_valid()
    }

    #[must_use]
    pub const fn distance_to_home(self, color: Color) -> u8 {
        64 - (match color {
            Color::Black => self.0,
            Color::Blue => self.0 + 48,
            Color::Green => self.0 + 32,
            Color::Red => self.0 + 16,
        } & 63)
            & 63
    }

    #[must_use]
    pub fn distance_to(self, other: Square) -> u8 {
        if self <= other {
            other.0 - self.0
        } else {
            64 - (self.0 - other.0)
        }
    }

    #[must_use]
    pub fn is_min(self) -> bool {
        self == Self::MIN
    }

    #[must_use]
    pub fn is_max(self) -> bool {
        self == Self::MAX
    }

    #[must_use]
    pub fn in_range(self, start: Square, end: Square) -> bool {
        if start < end {
            (start.0..=end.0).contains(&self.0)
        } else {
            (start.0..=63).contains(&self.0) || (0..=end.0).contains(&self.0)
        }
    }
}
