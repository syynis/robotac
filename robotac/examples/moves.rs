use mcts::{manager::Manager, policies::UCTPolicy};
use robotac::{board::Board, TacAI, TacEval};

fn main() {
    let mut mcts = Manager::new(Board::new_with_seed(0), TacAI, UCTPolicy(35.0), TacEval);
    println!("{:?}", mcts.tree().root_state());

    (0..24).for_each(|_| {
        if mcts.legal_moves().len() != 1 {
            mcts.playout_n_parallel(500_000, 8);
        }
        if let Some(best_move) = mcts.best_move() {
            mcts.print_root_legal_moves();
            println!("Make move {:?}", best_move);
            mcts.advance(&best_move);
        };
    });
    println!("{:?}", mcts.tree().root_state());
    mcts.print_knowledge();
}
