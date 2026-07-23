use biter::{Biter,MutBiter};
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

fn bit_iter(c: &mut Criterion) {
    let zend: Vec<u64> = (0..1000).rev().chain(core::iter::repeat(0).take(9000)).collect();
    c.bench_function("bit_iter", |b|
        b.iter(|| {
            let mut set_bits = 0;
            Biter::from(&zend).for_each(|bit| set_bits += bit as usize);
            black_box(set_bits);
        }));
}


fn bit_iter_mut(c: &mut Criterion) {
    let zend: Vec<u64> = (0..1000).rev().chain(core::iter::repeat(0).take(9000)).collect();
    c.bench_function("bit_iter_mut", |b|
        b.iter(|| {
            MutBiter::from(&mut zend.clone()).for_each(|bit| {*black_box(bit) = true});
        }));
}

fn popcnt(c: &mut Criterion) {
    let zend: Vec<u64> = (0..1000).rev().chain(core::iter::repeat(0).take(9000)).collect();
    c.bench_function("popcnt", |b|
        b.iter(|| {
            black_box(Biter::from(&zend).popcnt())
        })
    );
}

fn ctz(c: &mut Criterion) {
    let zend: Vec<u64> = (0..1000).rev().chain(core::iter::repeat(0).take(9000)).collect();
    c.bench_function("ctz", |b|
        b.iter(|| {
            black_box(Biter::from(&zend).ctz())
        })
    );
}

fn first_one(c: &mut Criterion) {
    let zend: Vec<u64> = core::iter::repeat(0).take(9999).chain(core::iter::repeat(!0).take(1)).collect();
    c.bench_function("first_one", |b|
        b.iter(|| {
            black_box(Biter::from(&zend).first_one())
        })
    );
}

fn first_zero(c: &mut Criterion) {
    let zend: Vec<u64> = core::iter::repeat(!0).take(9999).chain(core::iter::repeat(0).take(1)).collect();
    c.bench_function("first_zero", |b|
        b.iter(|| {
            black_box(Biter::from(&zend).first_zero())
        })
    );
}

criterion_group!(biters, bit_iter,bit_iter_mut);
criterion_group!(counters, popcnt,ctz);
criterion_group!(first, first_one,first_zero);
criterion_main!(biters,counters,first);
