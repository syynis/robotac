use std::time::Instant;

use mcts::{manager::Manager, policies::UCTPolicy};
use robotac::{board::Board, TacAI, TacEval};

fn main() {
    let mut mcts = Manager::new(Board::new_with_seed(1), TacAI, UCTPolicy(35.0), TacEval);
    println!("{:?}", mcts.tree().root_state());

    let before = Instant::now();
    // mcts.playout_n(2_500_000);
    mcts.playout_n_parallel(5_000_000, 8);
    let after = Instant::now();
    println!("playout in {}", (after - before).as_secs_f32());
    mcts.print_stats();
    (0..4).for_each(|_| {
        if let Some(best_move) = mcts.best_move() {
            mcts.print_root_legal_moves();
            println!("Make move {:?}", best_move);
            mcts.advance(&best_move);
        };
    });
    // mcts.playout_n(2_500_000);
    mcts.playout_n_parallel(5_000_000, 8);
    (0..15).for_each(|_| {
        if let Some(best_move) = mcts.best_move() {
            mcts.print_root_legal_moves();
            println!("Make move {:?}", best_move);
            mcts.advance(&best_move);
        };
    });
    println!("{:?}", mcts.tree().root_state());
    mcts.print_knowledge();
}
