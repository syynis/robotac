use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::seq::IteratorRandom;
use robotac::board::Board;
use tac_types::{ALL_COLORS, CARDS};

pub fn criterion_benchmark(criterion: &mut Criterion) {
    let mut rng = rand::thread_rng();
    let mut board = Board::new_with_seed(0);
    for color in ALL_COLORS {
        board.put_ball_in_play(color);
        board.move_ball(color.home(), color.home().add(4), color);
        board.put_ball_in_play(color);
        board.move_ball(color.home(), color.home().sub(4), color);
        board.put_ball_in_play(color);
        board.move_ball_to_goal(color.home(), 2, color);
    }
    criterion.bench_function("gen moves", |b| {
        b.iter(|| {
            for c in black_box(CARDS) {
                black_box(board.moves_for_card(tac_types::Color::Black, c));
            }
        });
    });
    criterion.bench_function("apply", |b| {
        b.iter(|| {
            let mut board = board.clone();
            black_box(for _ in 0..100 {
                let get_moves = &board.get_moves(board.current_player());
                let Some(mv) = get_moves.iter().choose(&mut rng) else {
                    // Game over
                    break;
                };
                board.play(mv);
            })
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(300).warm_up_time(Duration::from_secs(10));
    targets = criterion_benchmark
}
criterion_main!(benches);
