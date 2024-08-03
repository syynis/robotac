use crate::{square::Square, Card};

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

impl TacMove {
    pub fn new(card: Card, action: TacAction) -> Self {
        Self { card, action }
    }
}
