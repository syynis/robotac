use std::ops::{
    BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Sub, SubAssign,
};

use crate::square::Square;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
pub struct BitBoard(pub u64);

impl BitBoard {
    pub const EMPTY: Self = Self(0);
    pub const ONE: Self = Self(1);

    #[must_use]
    #[inline(always)]
    pub const fn is_empty(self) -> bool {
        self.0 == Self::EMPTY.0
    }

    #[must_use]
    #[inline(always)]
    pub const fn len(self) -> usize {
        self.0.count_ones() as usize
    }

    #[must_use]
    #[inline(always)]
    pub const fn has(self, square: Square) -> bool {
        !self.is_disjoint(square.bitboard())
    }

    #[must_use]
    #[inline(always)]
    pub const fn is_disjoint(self, other: BitBoard) -> bool {
        self.0 & other.0 == Self::EMPTY.0
    }

    #[must_use]
    #[inline(always)]
    pub const fn is_subset(self, other: BitBoard) -> bool {
        other.0 & self.0 == self.0
    }

    #[must_use]
    #[inline(always)]
    pub const fn is_superset(self, other: BitBoard) -> bool {
        other.is_subset(self)
    }

    #[must_use]
    #[inline(always)]
    pub const fn try_next_square(self) -> Option<Square> {
        if self.is_empty() {
            return None;
        }
        let index = self.0.trailing_zeros() as u8;
        Some(Square(index))
    }

    #[must_use]
    #[inline(always)]
    pub const fn next_square(self) -> Square {
        Square(self.0.trailing_zeros() as u8)
    }

    #[must_use]
    #[inline(always)]
    pub const fn iter(self) -> BitBoardIter {
        BitBoardIter(self)
    }

    #[must_use]
    #[inline(always)]
    pub const fn invert_trailing(self) -> Self {
        Self(self.0 - 1)
    }

    #[must_use]
    #[inline(always)]
    pub const fn rotate_right(self, n: u8) -> Self {
        Self(self.0.rotate_right(n as u32))
    }

    #[must_use]
    #[inline(always)]
    pub const fn rotate_left(self, n: u8) -> Self {
        Self(self.0.rotate_left(n as u32))
    }
}

pub struct BitBoardIter(BitBoard);

impl Iterator for BitBoardIter {
    type Item = Square;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let square = self.0.try_next_square();
        if let Some(square) = square {
            self.0 ^= square.bitboard();
        }
        square
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl IntoIterator for BitBoard {
    type Item = Square;
    type IntoIter = BitBoardIter;

    #[inline(always)]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl ExactSizeIterator for BitBoardIter {
    #[inline(always)]
    fn len(&self) -> usize {
        self.0.len()
    }
}

macro_rules! impl_math_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait for BitBoard {
            type Output = Self;

            #[inline(always)]
            fn $fn(self, rhs: Self) -> Self::Output {
                Self($trait::$fn(self.0, rhs.0))
            }
        }
    )*};
}
impl_math_ops! {
    BitAnd, bitand;
    BitOr, bitor;
    BitXor, bitxor;
}

macro_rules! impl_math_assign_ops {
    ($($trait:ident, $fn:ident;)*) => {$(
        impl $trait for BitBoard {
            #[inline(always)]
            fn $fn(&mut self, rhs: Self) {
                $trait::$fn(&mut self.0, rhs.0)
            }
        }
    )*};
}
impl_math_assign_ops! {
    BitAndAssign, bitand_assign;
    BitOrAssign, bitor_assign;
    BitXorAssign, bitxor_assign;
}

impl Sub for BitBoard {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        self & !rhs
    }
}

impl SubAssign for BitBoard {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Not for BitBoard {
    type Output = Self;

    #[inline(always)]
    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}
