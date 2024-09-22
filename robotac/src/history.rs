use crate::board::Board;
use serde::{Deserialize, Serialize};
use tac_types::TacMove;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct History {
    pub seed: u64,
    pub moves: Vec<TacMove>,
}

impl History {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Self {
            seed,
            moves: Vec::new(),
        }
    }

    #[must_use]
    pub fn board_with_history(&self) -> Board {
        let mut board = Board::new_with_seed(self.seed);
        for mv in &self.moves {
            board.play(mv);
        }
        board
    }
}
