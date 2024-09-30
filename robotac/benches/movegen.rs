use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use robotac::board::Board;
use tac_types::{ALL_COLORS, CARDS};

pub fn criterion_benchmark(criterion: &mut Criterion) {
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
            for p in black_box(ALL_COLORS) {
                for c in black_box(CARDS) {
                    black_box(board.moves_for_card(p, c));
                }
            }
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(300).measurement_time(Duration::from_secs(30));
    targets = criterion_benchmark
}
criterion_main!(benches);
