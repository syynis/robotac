#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_panics_doc,
    clippy::similar_names,
    clippy::struct_excessive_bools
)]
use board::Board;
use mcts::{policies::UCTPolicy, Evaluator, GameState, MCTS};
use tac_types::{Color, TacMove};

pub mod board;
pub mod deck;
pub mod eval;
pub mod hand;
pub mod knowledge;
pub mod movegen;
pub mod seven;

struct TacAI;
struct TacEval;

impl MCTS for TacAI {
    type State = Board;
    type Eval = TacEval;
    type Select = UCTPolicy;
}

impl Evaluator<TacAI> for TacEval {
    type StateEval = i64;

    fn state_eval_new(
        &self,
        state: &<TacAI as MCTS>::State,
        _handle: Option<mcts::search::SearchHandle<TacAI>>,
    ) -> Self::StateEval {
        state.eval()
    }

    fn eval_new(
        &self,
        state: &<TacAI as MCTS>::State,
        _moves: &mcts::MoveList<TacAI>,
        _handle: Option<mcts::search::SearchHandle<TacAI>>,
    ) -> (Vec<mcts::MoveEval<TacAI>>, Self::StateEval) {
        (Vec::new(), state.eval())
    }

    fn eval_existing(
        &self,
        _state: &<TacAI as MCTS>::State,
        existing: &Self::StateEval,
        _handle: mcts::search::SearchHandle<TacAI>,
    ) -> Self::StateEval {
        *existing
    }

    fn make_relative(&self, eval: &Self::StateEval, player: &mcts::Player<TacAI>) -> i64 {
        match player {
            Color::Black | Color::Green => *eval,
            Color::Blue | Color::Red => -*eval,
        }
    }
}

impl GameState for Board {
    type Move = TacMove;
    type Player = Color;
    type MoveList = Vec<Self::Move>;

    fn current_player(&self) -> Self::Player {
        self.current_player()
    }

    fn legal_moves(&self) -> Self::MoveList {
        self.get_moves(self.current_player())
    }

    fn make_move(&mut self, mv: &Self::Move) {
        self.play(mv);
    }

    fn randomize_determination(&mut self, _observer: Self::Player) {
        todo!()
    }
}
