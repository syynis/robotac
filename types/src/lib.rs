#![warn(clippy::pedantic)]
// NOTE We allow this here because all truncations are values
// returned from functions that depend on the number of bits on a u64
//  which can never exceed 64 which fits into u8
#![allow(clippy::cast_possible_truncation)]
pub mod bitboard;
pub mod card;
pub mod color;
pub mod home;
pub mod square;
pub mod tacmove;

pub use bitboard::*;
pub use card::*;
pub use color::*;
pub use home::*;
pub use square::*;
pub use tacmove::*;
