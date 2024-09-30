use serde::{Deserialize, Serialize};

use crate::{bitboard::BitBoard, color::Color};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash, Serialize, Deserialize,
)]
pub struct Square(pub u8);

impl Square {
    #[must_use]
    #[inline(always)]
    pub const fn bitboard(self) -> BitBoard {
        BitBoard(1 << self.0)
    }

    // Wrap square around range 0 to 63.
    // NOTE This only works if square value is 127 at most
    #[must_use]
    #[inline(always)]
    pub const fn make_valid(self) -> Self {
        Self(self.0 & 63)
    }

    #[must_use]
    #[inline(always)]
    pub const fn add(self, amount: u8) -> Self {
        Self(self.0 + amount).make_valid()
    }

    #[must_use]
    #[inline(always)]
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
    }

    #[must_use]
    pub fn distance_to(self, other: Square) -> u8 {
        if self <= other {
            other.0 - self.0
        } else {
            64 - (self.0 - other.0)
        }
    }
}
