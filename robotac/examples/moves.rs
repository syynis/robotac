use std::io;

use itertools::Itertools;
use robotac::board::Board;

fn main() {
    let mut board = Board::new_with_seed(0);
    let mut input = String::new();
    println!("{:?}", board);
    for (idx, mv) in board.get_moves(board.current_player()).iter().enumerate() {
        println!("{} {:?}", idx, mv);
    }
    loop {
        if io::stdin().read_line(&mut input).is_ok() {
            let i = input.strip_suffix('\n').unwrap().to_owned();
            if let Ok(number) = i.parse::<usize>() {
                let moves = board.get_moves(board.current_player());
                board.play(moves[number].clone());
                let hand = board.hand(board.current_player());
                println!("{:?}", hand);
                for (idx, mv) in board.get_moves(board.current_player()).iter().enumerate() {
                    println!("{} {:?}", idx, mv);
                }
            } else if i == "state" {
                println!("{:?}", board);
            } else if i == "moves" {
                for (idx, mv) in board.get_moves(board.current_player()).iter().enumerate() {
                    println!("{} {:?}", idx, mv);
                }
            }
            input.clear();
        }
    }
}
