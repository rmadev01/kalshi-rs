//! Benchmarks for orderbook operations.
//!
//! Run with: `cargo bench`

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use kalshi_rs::orderbook::Orderbook;
use kalshi_rs::types::order::Side;

fn bench_orderbook_delta(c: &mut Criterion) {
    let mut group = c.benchmark_group("orderbook_delta");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut book = Orderbook::new("BENCH");

            // Pre-populate with some levels
            for i in 1..=size {
                book.set_level(i as u8 % 99 + 1, 100, Side::Yes);
            }

            b.iter(|| {
                // Simulate a typical delta
                book.apply_delta(black_box(50), black_box(10), black_box(Side::Yes));
            });
        });
    }

    group.finish();
}

fn bench_orderbook_best_bid(c: &mut Criterion) {
    let mut group = c.benchmark_group("orderbook_best_bid");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            let mut book = Orderbook::new("BENCH");

            // Pre-populate with levels
            for i in 1..=size {
                book.set_level(i as u8 % 99 + 1, 100, Side::Yes);
            }

            b.iter(|| {
                black_box(book.best_bid());
            });
        });
    }

    group.finish();
}

fn bench_orderbook_spread(c: &mut Criterion) {
    let mut book = Orderbook::new("BENCH");

    // Set up a realistic book
    for i in 1..=50 {
        book.set_level(40 + i as u8 % 10, 100 * i, Side::Yes);
        book.set_level(50 + i as u8 % 10, 100 * i, Side::No);
    }

    c.bench_function("orderbook_spread", |b| {
        b.iter(|| {
            black_box(book.spread());
        });
    });
}

criterion_group!(
    benches,
    bench_orderbook_delta,
    bench_orderbook_best_bid,
    bench_orderbook_spread
);
criterion_main!(benches);
