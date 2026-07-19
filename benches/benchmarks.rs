use biter::{Biter,MutBiter};
fn main() {divan::main();}

use divan::{Bencher, black_box};

#[divan::bench]
fn bit_iter(bencher: Bencher) {
    bencher
        .with_inputs(|| {
             let zend: Vec<u64> = (0..1000).rev().chain(core::iter::repeat(0).take(9000)).collect();
             zend
        })
        .bench_refs(|zend| {
            let mut set_bits = 0;
            Biter::from(zend).for_each(|bit| set_bits += bit as usize);
            black_box(set_bits);
        });
}

#[divan::bench]
fn bit_iter_mut(bencher: Bencher) {
    bencher
        .with_inputs(|| {
             let zend: Vec<u64> = (0..1000).rev().chain(core::iter::repeat(0).take(9000)).collect();
             zend
        })
        .bench_values(|zend| {
            MutBiter::from(zend).for_each(|mut bit| {*black_box(bit) = true;
            });
        });
}
