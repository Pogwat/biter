#![no_std]
#[doc = include_str!("../README.md")]
use core::marker::PhantomData;
use bit_operations::{BitOps,MutBitProxy};
macro_rules! biterators {
    (name:$name:ident, item:$item:ty, bit_method:$bit_method:ident, $((S:$($sp:tt)*),)?to_slice:$to_slice:ident, ptr_ty:$ptr_ty:tt  $(, lock:$lock:tt)? ) => {
        /// The Bit Iterator
        pub struct $name<'short,ElementType> {
            current_pointer: *$ptr_ty ElementType,
            remaining_bits: usize,
            bit_position:u8,
            _bitlife: PhantomData<&'short $($lock)? ElementType>,
        }
        impl<'short, ElementType: BitOps> Iterator for $name<'short, ElementType> {
            type Item = $item;
            fn next(&mut self) -> Option<Self::Item> {
                if self.remaining_bits!=0 {
                    let bit = unsafe {(*self.current_pointer).$bit_method(self.bit_position) };
                    self.bit_position+=1;
                    self.remaining_bits-=1;
                    if self.bit_position==ElementType::BITS as u8 {
                        self.bit_position=0;
                        unsafe {self.current_pointer = self.current_pointer.add(1)};
                    }
                    Some(bit)
                } else {None}
            }

            //int + func . provide accum and bit to func
            fn fold<B, F: FnMut(B, Self::Item) -> B>(mut self, init: B, mut f: F) -> B {
                let mut accum = init;
                while self.remaining_bits!=0 {
                    let wbend:u8 = (ElementType::BITS as usize).min(self.remaining_bits) as u8;
                    for bit_pos in (self.bit_position..wbend) {
                        let bit = unsafe {(*self.current_pointer).$bit_method(bit_pos) };
                        accum = f(accum, bit);
                    }
                    self.remaining_bits-=(wbend-self.bit_position) as usize;
                    self.bit_position=0;
                    unsafe {self.current_pointer = self.current_pointer.add((self.remaining_bits != 0) as usize)} //Illegal pointer at end of iterator, only inc if there are remeaning bits
                }
                accum
            }
        }
        impl<'short, ElementType: BitOps> $name<'short,ElementType>{
            /// Biterator from a start pointer, start bit and remaining bits
            pub unsafe fn new(current_pointer:*$ptr_ty ElementType, bit_position:u8, remaining_bits:usize) -> Self {Self {current_pointer, bit_position, remaining_bits, _bitlife: PhantomData} }
            /// Remaining bits to iterate over (self.remaining_bits)
            pub fn remaining_bits(&self) -> usize {self.remaining_bits}
            /// Biterator from a number
            pub fn from_num(s:&'short $($lock)? ElementType) -> Self { unsafe {Self::new(s as *$ptr_ty ElementType,0,ElementType::BITS as usize)}}
            /// Add (or subtract) a amount to remaining_bits, resizing the iterator
            pub unsafe fn uncheked_resize_bits(&mut self, resize_amount:isize) {
                self.remaining_bits=self.remaining_bits.wrapping_add_signed(resize_amount) // Wraps
            }
        }

        /// Biterator from anything that can be sliced (collections)
        impl <'short,ElementType: BitOps,S:AsRef<[ElementType]>+$($($sp)*)? > From<S> for $name<'short,ElementType> {
            fn from($($lock)? s:S) -> Self {unsafe {Self::new(s.$to_slice() as *$ptr_ty [ElementType] as *$ptr_ty ElementType,0,s.as_ref().len()*(ElementType::BITS as usize)) }}
        }
    }
}
biterators!(name:Biter,item:bool,bit_method:get_bit,to_slice:as_ref, ptr_ty:const);
biterators!(name:MutBiter,item:MutBitProxy<'short,ElementType>,bit_method:mut_bit,(S:AsMut<[ElementType]>),to_slice:as_mut, ptr_ty:mut, lock:mut);
