use enum_map::Enum;
use serde::{Deserialize, Serialize};

pub const NUM_CARDS: usize = 18;
pub const CARDS: [Card; NUM_CARDS] = [
    Card::One,
    Card::Two,
    Card::Three,
    Card::Four,
    Card::Five,
    Card::Six,
    Card::Seven,
    Card::Eight,
    Card::Nine,
    Card::Ten,
    Card::Twelve,
    Card::Thirteen,
    Card::Trickster,
    Card::Jester,
    Card::Angel,
    Card::Devil,
    Card::Warrior,
    Card::Tac,
];

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Enum, PartialOrd, Ord, Serialize, Deserialize,
)]
#[repr(u8)]
pub enum Card {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Twelve,
    Thirteen,
    Trickster,
    Jester,
    Angel,
    Devil,
    Warrior,
    Tac,
}

impl Card {
    #[must_use]
    pub const fn amount(self) -> u8 {
        match self {
            Card::Seven => 8,
            Card::Jester | Card::Angel | Card::Devil | Card::Warrior => 1,
            Card::Tac => 4,
            Card::One | Card::Thirteen => 9,
            _ => 7,
        }
    }

    #[must_use]
    pub const fn from_steps(steps: u8) -> Option<Card> {
        match steps {
            1 => Some(Card::One),
            2 => Some(Card::Two),
            3 => Some(Card::Three),
            5 => Some(Card::Five),
            6 => Some(Card::Six),
            7 => Some(Card::Seven),
            8 => Some(Card::Eight),
            9 => Some(Card::Nine),
            10 => Some(Card::Ten),
            12 => Some(Card::Twelve),
            13 => Some(Card::Thirteen),
            _ => None,
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            Card::One => "1",
            Card::Two => "2",
            Card::Three => "3",
            Card::Four => "4",
            Card::Five => "5",
            Card::Six => "6",
            Card::Seven => "7",
            Card::Eight => "8",
            Card::Nine => "9",
            Card::Ten => "10",
            Card::Twelve => "12",
            Card::Thirteen => "13",
            Card::Trickster => "Trickster",
            Card::Jester => "Jester",
            Card::Angel => "Angel",
            Card::Devil => "Devil",
            Card::Warrior => "Warrior",
            Card::Tac => "Tac",
        }
    }

    #[must_use]
    pub fn is_simple(self) -> Option<u8> {
        match self {
            Card::One => Some(1),
            Card::Two => Some(2),
            Card::Three => Some(3),
            Card::Five => Some(5),
            Card::Six => Some(6),
            Card::Eight => Some(8),
            Card::Nine => Some(9),
            Card::Ten => Some(10),
            Card::Twelve => Some(12),
            Card::Thirteen => Some(13),
            _ => None,
        }
    }
}
