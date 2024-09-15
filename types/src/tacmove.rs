use std::fmt::Display;

use smallvec::SmallVec;

use crate::{square::Square, Card, Color};

#[derive(Debug, Clone, PartialEq)]
pub enum TacAction {
    Step { from: Square, to: Square },
    // TODO HomeSquare type
    StepHome { from: u8, to: u8 },
    StepInHome { from: Square, to: u8 },
    Switch { target1: Square, target2: Square },
    Enter,
    Suspend,
    Jester,
    Devil,
    Warrior { from: Square, to: Square },
    Discard,
    AngelEnter,
    Trade,
    SevenSteps { steps: Vec<TacAction> },
}

impl Display for TacAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let _ = match self {
            TacAction::Step { from, to } => write!(f, "Step {} {}", from.0, to.0),
            TacAction::StepHome { from, to } => write!(f, "Home {} {}", from, to),
            TacAction::StepInHome { from, to } => write!(f, "In home {} {}", from.0, to),
            TacAction::Switch { target1, target2 } => {
                write!(f, "Switch {} {}", target1.0, target2.0)
            }
            TacAction::Warrior { from, to } => write!(f, "Warrior {} {}", from.0, to.0),
            TacAction::SevenSteps { steps } => {
                for (idx, s) in steps.iter().enumerate() {
                    if idx == steps.len() - 1 {
                        write!(f, "{}", s);
                    } else {
                        write!(f, "{} | ", s);
                    }
                }
                write!(f, "")
            }
            _ => write!(f, "{:?}", self),
        };
        Ok(())
    }
}

// TODO this can probably fit into 32 bits if we are very clever
#[derive(Debug, Clone, PartialEq)]
pub struct TacMove {
    pub card: Card,
    pub action: TacAction,
    pub played_for: Color,
}

impl Display for TacMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:?} {} {:?}", self.card, self.action, self.played_for)
    }
}

impl TacMove {
    pub fn new(card: Card, action: TacAction, played_for: Color) -> Self {
        Self {
            card,
            action,
            played_for,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TacMoveResult {
    Capture(Color),
    SevenCaptures(SmallVec<Color, 7>),
}
