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

fn short_bit_iter(c: &mut Criterion) {
    let zend: Vec<u64> = (0..5).rev().collect();
    c.bench_function("short_bit_iter", |b|
        b.iter(|| {
            let mut set_bits = 0;
            Biter::from(&zend).for_each(|bit| set_bits += bit as usize);
            black_box(set_bits);
        }));
}

fn bit_iter_next(c: &mut Criterion) {
    let zend: Vec<u64> = (0..1000).rev().chain(core::iter::repeat(0).take(9000)).collect();
    let mut biter = Biter::from(&zend);
    c.bench_function("bit_iter_next", |b|
        b.iter(|| {
            let mut set_bits = 0;
            while let Some(bit) = biter.next() {
                set_bits += bit as usize;
            }
            black_box(set_bits);
            biter=Biter::from(&zend);
        }));
}


fn bit_iter_next_back(c: &mut Criterion) {
    let zend: Vec<u64> = (0..1000).rev().chain(core::iter::repeat(0).take(9000)).collect();
    let mut biter = Biter::from(&zend);
    c.bench_function("bit_iter_next_back", |b|
        b.iter(|| {
            let mut set_bits = 0;
            while let Some(bit) = biter.next_back() {
                set_bits += bit as usize;
            }
            black_box(set_bits);
            biter=Biter::from(&zend);
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
criterion_group!(biters, bit_iter,bit_iter_mut,short_bit_iter,bit_iter_next,bit_iter_next_back);
criterion_group!(counters, popcnt,ctz);
criterion_group!(first, first_one,first_zero);
criterion_main!(biters,counters,first);
