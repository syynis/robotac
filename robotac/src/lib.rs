#![warn(clippy::pedantic)]
#![allow(
    clippy::missing_panics_doc,
    clippy::similar_names,
    clippy::struct_excessive_bools
)]
use board::Board;
use knowledge::Knowledge;
use mcts::{policies::UCTPolicy, Evaluator, GameState, MCTS};
use tac_types::{Color, TacMove};

pub mod board;
pub mod eval;
pub mod history;
pub mod knowledge;
pub mod movegen;
pub mod seven;

pub struct TacAI;
pub struct TacEval;

impl MCTS for TacAI {
    type State = Board;
    type Eval = TacEval;
    type Select = UCTPolicy;
}

impl Evaluator<TacAI> for TacEval {
    type StateEval = i64;

    fn eval_new(
        &self,
        state: &<TacAI as MCTS>::State,
        _handle: Option<mcts::search::SearchHandle<TacAI>>,
    ) -> Self::StateEval {
        state.eval2()
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
    type Knowledge = Knowledge;

    fn current_player(&self) -> Self::Player {
        self.current_player()
    }

    fn legal_moves(&self) -> Self::MoveList {
        self.get_moves(self.current_player())
    }

    fn make_move(&mut self, mv: &Self::Move) {
        self.play(mv);
    }

    fn randomize_determination(&mut self, observer: Self::Player, knowledge: &Self::Knowledge) {
        self.redetermine(observer, knowledge);
    }

    fn update_knowledge(&self, mv: &Self::Move, knowledge: &mut Self::Knowledge) {
        knowledge.update_after_move(mv, self);
    }

    fn knowledge_from_state(&self, observer: Self::Player) -> Self::Knowledge {
        Knowledge::new_from_board(observer, self)
    }
}
