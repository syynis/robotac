use std::fmt::Display;

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TacMove {
    pub card: Card,
    pub action: TacAction,
    pub played_for: Color,
    pub played_by: Color,
}

impl Display for TacMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {} {:?}", self.card, self.action, self.played_for)
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

pub enum PackedTacMoveResult {
    Capture(Color),
    // Square -> 6 bits
    // Color -> 2 bits
    // (6 + 2) * 7 -> u64
    // half the size of unpacked
    SevenCaptures(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum TacMoveResult {
    Capture(Color),
    SevenCaptures(SmallVec<(Square, Color), 7>),
}
