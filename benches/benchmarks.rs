use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

use biter::{Biter,MutBiter};
fn criterion_benchmark(c: &mut Criterion) {
    //last one is about 9000 elements from the end
    let zend: Vec<u64> = (0..1000).rev().chain(core::iter::repeat(0).take(9000)).collect();

    // 4.8814 µs per iter
    c.bench_function("biter", |b| {
        b.iter(|| {
            let mut set_bits = 0;
            Biter::from(&zend).for_each(|bit| set_bits+=bit as usize);
            })
    });

    // 24.660 µs per iter
    c.bench_function("biter_mut", |b| {
        b.iter(|| {MutBiter::from(zend.clone()).for_each(|mut bit| *bit=true)})
    });


}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
