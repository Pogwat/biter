use biter::{Biter,MutBiter};
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;
use bit_operations::BitOps;

fn bit_iter(c: &mut Criterion) {
    let zend: Vec<u64> = (0..1000).rev().chain(core::iter::repeat(0).take(9000)).collect();
    c.bench_function("bit_iter", |b|
        b.iter(|| {
            let mut set_bits = 0;
            Biter::from(&zend).for_each(|bit| set_bits += bit as usize);
            black_box(set_bits);
        }));
}

fn normal_iter(c: &mut Criterion) {
    let zend: Vec<u64> = (0..1000).rev().chain(core::iter::repeat(0).take(9000)).collect();
    c.bench_function("normal_iter", |b|
        b.iter(|| {
            let mut set_bits = 0;
            zend.iter().for_each(|word| set_bits+=word.count_ones() as usize);
            black_box(set_bits);
        }));
}

fn normal_biter(c: &mut Criterion) {
    let zend: Vec<u64> = (0..1000).rev().chain(core::iter::repeat(0).take(9000)).collect();
    c.bench_function("normal_biter", |b|
        b.iter(|| {
            let mut set_bits = 0;
            for word in &zend {
                for bit in 0..64 {
                    set_bits+=word.get_bit(bit) as usize;
                }
            }
            black_box(set_bits);
        }));
}

fn normal_first_one(c: &mut Criterion) {
    let zend: Vec<u64> = core::iter::repeat(0).take(9999).chain(core::iter::repeat(!0).take(1)).collect();
    c.bench_function("normal_first_one", |b|
        b.iter(|| {
            black_box(zend.iter().position(|word| word.first_one(&(0..)).is_some()));
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

fn last_one(c: &mut Criterion) {
    let zend: Vec<u64> = core::iter::repeat(!0).take(1).chain(core::iter::repeat(0).take(9999)).collect();
    c.bench_function("last_one", |b|
        b.iter(|| {
            black_box(Biter::from(&zend).last_one())
        })
    );
}

fn last_zero(c: &mut Criterion) {
    let fend: Vec<u64> = core::iter::repeat(0).take(1).chain(core::iter::repeat(!0).take(9999)).collect();
    c.bench_function("last_zero", |b|
        b.iter(|| {
            black_box(Biter::from(&fend).last_zero())
        })
    );
}

criterion_group!(biters, bit_iter,bit_iter_mut,short_bit_iter,bit_iter_next,bit_iter_next_back);
criterion_group!(counters, popcnt,ctz);
criterion_group!(first, first_one,first_zero);
criterion_group!(last, last_one,last_zero);
criterion_group!(normal, normal_iter,normal_biter,normal_first_one);
criterion_main!(normal,biters,counters,first,last);
