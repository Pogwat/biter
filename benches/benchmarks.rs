use std::hint::black_box;
use biter::{Biter,MutBiter};
fn main() {divan::main();}

#[divan::bench]
fn bit_iter() {
    let zend: Vec<u64> = (0..1000).rev().chain(core::iter::repeat(0).take(9000)).collect();
    let mut set_bits=0;
    Biter::from(&zend).for_each(|bit| set_bits+=bit as usize);
}

#[divan::bench]
fn bit_iter_mut() {
    let zend: Vec<u64> = (0..1000).rev().chain(core::iter::repeat(0).take(9000)).collect();
    MutBiter::from(zend).for_each(|mut bit| *bit=true)
}
