use std::fmt::Display;

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::{square::Square, BitBoard, Card, Color};

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
    // SevenSteps2 { steps: Vec<(SevenAction, bool)> },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SevenAction {
    Step { from: Square, dist: u8 },
    StepHome { from: Square, to: u8 },
    StepInHome { from: u8, to: u8 },
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
    Seven(u64),
}

// Normal
// 0000 0000 0000 0000 0000 0000 0000 0000
//                                  - ---- Card
//                                --       Played for
//                             - -         Played by
//                      --- ---            From
//              - ---- -                   To
//         - ---                           Action
// Seven
// 0000 0000 0000 0000 0000 0000 0000 0000
//                                       - Card
//                                     --  Played for
//                                  - -    Played by
//                            -- ---       Played by
//                                         Played by
//                                         Played by
//                                         Played by
//                                         Played by
impl PackedTacMove {
    const PLAYED_FOR: usize = 5;
    const PLAYED_BY: usize = 7;
    const FROM: usize = 9;
    const TO: usize = 15;
    const ACTION: usize = 21;
    const SQUARE_SZ: usize = 6;
    pub fn new(card: Card, action: TacAction, played_for: Color, played_by: Color) -> Self {
        assert!(!matches!(action, TacAction::SevenSteps { .. }));
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
            TacAction::SevenSteps { .. } => unreachable!(),
        };
        res |= action_id << Self::ACTION;
        PackedTacMove::Normal(res)
    }

    pub fn new_seven(
        card: Card,
        actions: Vec<(SevenAction, bool)>,
        played_for: Color,
        played_by: Color,
    ) -> Self {
        let mut res: u64 = 0;
        assert!(matches!(card, Card::Seven | Card::Tac));
        if matches!(card, Card::Tac) {
            res |= 1;
        }
        res |= (played_for as u64) << 1;
        res |= (played_by as u64) << 3;
        for (idx, (action, for_partner)) in actions.iter().cloned().enumerate() {
            match action {
                SevenAction::Step { from, dist } => {
                    res |= 0b01 << (idx as u64 * 6 + 5);
                    // TODO
                }
                SevenAction::StepHome { from, to } => {
                    res |= 0b10 << (idx as u64 * 6 + 5);
                    // TODO
                }
                SevenAction::StepInHome { from, to } => {
                    res |= 0b11 << (idx as u64 * 6 + 5);
                    // TODO
                }
            }
            if for_partner {
                res |= 1 << (idx * 6 + 2 + 5);
            }
        }
        PackedTacMove::Seven(res)
    }

    pub fn card(&self) -> Card {
        match self {
            // This is safe because Card has 18 entries which fits into 5 bits
            PackedTacMove::Normal(m) => unsafe {
                std::mem::transmute::<u8, Card>((m & 0b11111) as u8)
            },
            PackedTacMove::Seven(m) => {
                if m & 1 == 0 {
                    Card::Seven
                } else {
                    Card::Tac
                }
            }
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
            PackedTacMove::Seven(m) => {
                let mut steps = Vec::new();
                let mut m = m;
                let extract_move = |s: &mut u64| -> Option<(SevenAction, bool)> {
                    let kind = *s & 0b11;
                    *s >>= 2;
                    let partner = *s & 0b1;
                    *s >>= 1;
                    let action = match kind {
                        0 => None,
                        1 => {
                            let data = *s & 0b111111111;
                            *s >>= 9;
                            let from = data & 0b111111;
                            let dist = data >> 6;
                            Some(SevenAction::Step { from, dist })
                        }
                        2 => {
                            let data = *s & 0b11111111;
                            *s >>= 8;
                            let from = data & 0b111111;
                            let to = data >> 6;
                            Some(SevenAction::StepHome { from, to })
                        }
                        3 => {
                            let data = *s & 0b1111;
                            *s >>= 4;
                            let from = data & 0b11;
                            let to = data >> 2;
                            Some(SevenAction::StepInHome { from, to })
                        }
                        _ => unreachable!(),
                    }?;
                    Some((action, partner > 0))
                };

                while let Some(x) = extract_moves(&mut m) {
                    // steps.push(x);
                }

                TacAction::SevenSteps { steps }
            }
        }
    }

    pub fn played_for(&self) -> Color {
        match self {
            // This is safe because Color has 4 entries which fits into 2 bits
            PackedTacMove::Normal(m) => unsafe {
                std::mem::transmute::<u8, Color>((m >> Self::PLAYED_FOR & 0b11) as u8)
            },
            PackedTacMove::Seven(m) => unsafe {
                std::mem::transmute::<u8, Color>((m >> 1 & 0b11) as u8)
            },
        }
    }

    pub fn played_by(&self) -> Color {
        match self {
            // This is safe because Color has 4 entries which fits into 2 bits
            PackedTacMove::Normal(m) => unsafe {
                std::mem::transmute::<u8, Color>((m >> Self::PLAYED_BY & 0b11) as u8)
            },
            PackedTacMove::Seven(m) => unsafe {
                std::mem::transmute::<u8, Color>((m >> 3 & 0b11) as u8)
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
    #[test]
    fn packed_seven() {
        let packed = PackedTacMove::new_seven(
            Card::Seven,
            vec![
                (SevenAction::Step, 3, Color::Black),
                (SevenAction::Step, 3, Color::Black),
            ],
            Color::Black,
            Color::Green,
        );
    }
}
