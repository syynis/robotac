use std::fmt::Display;

use smallvec::SmallVec;

use crate::{square::Square, Card, Color};

#[derive(Debug, Clone, PartialEq, Hash)]
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

#[derive(Debug, Clone, PartialEq, Hash)]
pub struct TacMove {
    pub card: Card,
    pub action: TacAction,
}

impl Display for TacMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} : {:?}", self.card, self.action)
    }
}

impl TacMove {
    pub fn new(card: Card, action: TacAction) -> Self {
        Self { card, action }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TacMoveResult {
    Capture(Color),
    SevenCaptures(SmallVec<Color, 7>),
}
