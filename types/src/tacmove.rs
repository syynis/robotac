use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{square::Square, Card, Color};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TacAction {
    Step { from: Square, to: Square },
    // TODO HomeSquare type
    StepHome { from: u8, to: u8 },
    StepInHome { from: Square, to: u8 },
    Trickster { target1: Square, target2: Square },
    Enter,
    Suspend,
    Jester,
    Devil,
    Warrior { from: Square, to: Square },
    Discard,
    Trade,
    SevenSteps { steps: Vec<TacAction> },
}

impl Display for TacAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TacAction::Step { from, to } => {
                write!(f, "Step {} {}", from.0, to.0)?;
            }
            TacAction::StepHome { from, to } => {
                write!(f, "Home {from} {to}")?;
            }
            TacAction::StepInHome { from, to } => {
                write!(f, "In home {} {}", from.0, to)?;
            }
            TacAction::Trickster { target1, target2 } => {
                write!(f, "Switch {} {}", target1.0, target2.0)?;
            }
            TacAction::Warrior { from, to } => {
                write!(f, "Warrior {} {}", from.0, to.0)?;
            }
            TacAction::SevenSteps { steps } => {
                for (idx, s) in steps.iter().enumerate() {
                    if idx == steps.len() - 1 {
                        write!(f, "{s}")?;
                    } else {
                        write!(f, "{s} | ")?;
                    }
                }
            }
            _ => {
                write!(f, "{self:?}")?;
            }
        };
        Ok(())
    }
}

pub enum PackedTacMove {
    // pub card: Card,
    // 5 bits
    // pub action: TacAction,
    // 16 bits
    // from, to -> 12 bits
    // 13 variants -> 4 bits
    // total 12
    // pub played_for: Color,
    // 2 bits
    // ---
    // 5 + 16 + 2 => 23 -> u32
    // PROBLEM -> Seven
    // Seven needs 4 * (12 + 2)
    // 12 -> from, to
    // 2 -> color
    // -> 56 bits
    // half the size of unpacked
    // IDEA
    // Instead of storing square positions just store move amount
    // At most 7 -> 3 bits
    // For 4 moves that makes 12 bits
    // For each move 1 bit if move is for partner
    // For each move 1 bit if move goes in home
    // -> 12 + 4 + 4 -> 20 bits
    // This requires us to sort the moves by position and location
    Normal(u32),
    Seven(u32),
}

// 0000 0000 0000 0000 0000 0000 0000 0000
//                                  - ---- Card
//                                --       Played for
//                             - -         Played by
//                      --- ---            From
//              - ---- -                   To
//         - ---                           Action
impl PackedTacMove {
    const PLAYED_FOR: usize = 5;
    const PLAYED_BY: usize = 7;
    const FROM: usize = 9;
    const TO: usize = 15;
    const ACTION: usize = 21;
    const SQUARE_SZ: usize = 6;
    pub fn new(card: Card, action: TacAction, played_for: Color, played_by: Color) -> Self {
        let mut res: u32 = 0;
        res |= card as u32;
        res |= (played_for as u32) << Self::PLAYED_FOR;
        res |= (played_by as u32) << Self::PLAYED_BY;
        let square_into_bits =
            |from, to| (((to as u32) << Self::SQUARE_SZ) | from as u32) << Self::FROM;
        let action_id = match action {
            TacAction::Step { from, to } => {
                res |= square_into_bits(from.0, to.0);
                0
            }
            TacAction::StepHome { from, to } => {
                res |= square_into_bits(from, to);
                1
            }
            TacAction::StepInHome { from, to } => {
                res |= square_into_bits(from.0, to);
                2
            }
            TacAction::Trickster { target1, target2 } => {
                res |= square_into_bits(target1.0, target2.0);
                3
            }
            TacAction::Enter => 4,
            TacAction::Suspend => 5,
            TacAction::Jester => 6,
            TacAction::Devil => 7,
            TacAction::Warrior { from, to } => {
                res |= square_into_bits(from.0, to.0);
                8
            }
            TacAction::Discard => 9,
            TacAction::Trade => 10,
            TacAction::SevenSteps { ref steps } => 11,
        };
        if matches!(action, TacAction::SevenSteps { ref steps }) {
            return PackedTacMove::Seven(0);
        }
        res |= action_id << Self::ACTION;
        PackedTacMove::Normal(res)
    }

    pub fn card(&self) -> Card {
        match self {
            // This is safe because Card has 18 entries which fits into 5 bits
            PackedTacMove::Normal(m) | PackedTacMove::Seven(m) => unsafe {
                std::mem::transmute::<u8, Card>((m & 0b11111) as u8)
            },
        }
    }

    pub fn action(&self) -> TacAction {
        match self {
            PackedTacMove::Normal(m) => {
                let action = (m >> Self::ACTION) & 0b1111;
                let from = ((m >> Self::FROM) & 0b11111) as u8;
                let to = ((m >> Self::TO) & 0b11111) as u8;
                match action {
                    0 => TacAction::Step {
                        from: Square(from),
                        to: Square(to),
                    },
                    1 => TacAction::StepHome { from, to },
                    2 => TacAction::StepInHome {
                        from: Square(from),
                        to,
                    },
                    3 => TacAction::Trickster {
                        target1: Square(from),
                        target2: Square(to),
                    },
                    4 => TacAction::Enter,
                    5 => TacAction::Suspend,
                    6 => TacAction::Jester,
                    7 => TacAction::Devil,
                    8 => TacAction::Warrior {
                        from: Square(from),
                        to: Square(to),
                    },
                    9 => TacAction::Discard,
                    10 => TacAction::Trade,
                    _ => unreachable!(),
                }
            }
            // TODO
            PackedTacMove::Seven(m) => TacAction::SevenSteps { steps: Vec::new() },
        }
    }

    pub fn played_for(&self) -> Color {
        match self {
            // This is safe because Color has 4 entries which fits into 2 bits
            PackedTacMove::Normal(m) | PackedTacMove::Seven(m) => unsafe {
                std::mem::transmute::<u8, Color>((m >> Self::PLAYED_FOR & 0b11) as u8)
            },
        }
    }

    pub fn played_by(&self) -> Color {
        match self {
            // This is safe because Color has 4 entries which fits into 2 bits
            PackedTacMove::Normal(m) | PackedTacMove::Seven(m) => unsafe {
                std::mem::transmute::<u8, Color>((m >> Self::PLAYED_BY & 0b11) as u8)
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TacMove {
    pub card: Card,
    pub action: TacAction,
    pub played_for: Color,
    pub played_by: Color,
}

impl Display for TacMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {} {:?} {:?}",
            self.card, self.action, self.played_for, self.played_by
        )
    }
}

impl TacMove {
    #[must_use]
    pub fn new(card: Card, action: TacAction, played_for: Color, played_by: Color) -> Self {
        Self {
            card,
            action,
            played_for,
            played_by,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed() {
        let packed = PackedTacMove::new(
            Card::Three,
            TacAction::Step {
                from: 18.into(),
                to: 21.into(),
            },
            Color::Black,
            Color::Green,
        );
        if let PackedTacMove::Normal(p) = packed {
            println!("{:032b}", p);
        }
        assert_eq!(packed.card(), Card::Three);
        assert_eq!(packed.played_for(), Color::Black);
        assert_eq!(packed.played_by(), Color::Green);
        assert_eq!(
            packed.action(),
            TacAction::Step {
                from: 18.into(),
                to: 21.into()
            }
        );
    }
}
