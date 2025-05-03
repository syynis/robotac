use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rand::Rng;
use robotac::board::Board;

pub fn criterion_benchmark(criterion: &mut Criterion) {
    let mut rng = rand::thread_rng();
    criterion.bench_function("eval", |b| {
        b.iter(|| {
            let board = Board::new_random_state(rng.gen());
            board.eval();
        });
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(300).warm_up_time(Duration::from_secs(10));
    targets = criterion_benchmark
}
criterion_main!(benches);
