use crate::{bitboard::BitBoard, color::Color};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct Square(pub u8);

impl Square {
    pub const fn bitboard(self) -> BitBoard {
        BitBoard(1 << self.0)
    }

    // Wrap square around range 0 to 63.
    // NOTE This only works if square value is 127 at most
    pub const fn make_valid(self) -> Self {
        Self(self.0 & 63)
    }

    pub const fn add(self, amount: u8) -> Self {
        Self(self.0 + amount).make_valid()
    }

    pub const fn sub(self, amount: u8) -> Self {
        Self(self.0 + 64 - amount).make_valid()
    }

    pub const fn relative_to(self, color: Color) -> Self {
        // TODO
        // Pass and respect color order.
        // For now hardcoded which means the teams are black/green and blue/red
        match color {
            Color::Black => self,
            Color::Blue => Self(self.0 + 48).make_valid(),
            Color::Green => Self(self.0 + 32).make_valid(),
            Color::Red => Self(self.0 + 16).make_valid(),
        }
    }

    pub const fn distance_to_home(self, color: Color) -> u8 {
        64 - match color {
            Color::Black => self.0,
            Color::Blue => self.0 + 48,
            Color::Green => self.0 + 32,
            Color::Red => self.0 + 16,
        }
    }

    pub fn between_mask(self, other: Self) -> BitBoard {
        if self < other {
            other.bitboard().invert_trailing()
                & !(self.bitboard().invert_trailing() | self.bitboard())
        } else {
            !(self.bitboard().invert_trailing() | self.bitboard())
                & !(other.bitboard().invert_trailing() | other.bitboard())
        }
    }
}
