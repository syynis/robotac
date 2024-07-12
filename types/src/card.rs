use enum_map::Enum;

pub const CARDS: [Card; 18] = [
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
    Card::Juggler,
    Card::Jester,
    Card::Angel,
    Card::Devil,
    Card::Warrior,
    Card::Tac,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Enum, PartialOrd, Ord)]
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
    Juggler,
    Jester,
    Angel,
    Devil,
    Warrior,
    Tac,
}

impl Card {
    pub fn count(&self) -> u8 {
        match self {
            Card::Seven => 8,
            Card::Jester | Card::Angel | Card::Devil | Card::Warrior => 1,
            Card::Tac => 4,
            Card::One | Card::Thirteen => 9,
            _ => 7,
        }
    }

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
            Card::Juggler => "Juggler",
            Card::Jester => "Jester",
            Card::Angel => "Angel",
            Card::Devil => "Devil",
            Card::Warrior => "Warrior",
            Card::Tac => "Tac",
        }
    }

    pub fn is_simple(&self) -> Option<u8> {
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

    pub fn can_leave_house(&self) -> bool {
        matches!(self, Card::One | Card::Thirteen)
    }
}
