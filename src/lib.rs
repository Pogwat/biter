#![no_std]
#[doc = include_str!("../README.md")]
use core::marker::PhantomData;
use bit_operations::{BitOps,MutBitProxy};
use core::ops::{Range,ControlFlow};
macro_rules! biterators {
    (name:$name:ident, item:$item:ty, bit_method:$bit_method:ident, $((S:$($sp:tt)*),)?to_slice:$to_slice:ident, ptr_ty:$ptr_ty:tt  $(, lock:$lock:tt)? ) => {
        /// The Bit Iterator
        pub struct $name<'long,ElementType> {
            current_pointer: *$ptr_ty ElementType,
            remaining_bits: usize,
            bit_position:u8,
            _slicelife: PhantomData<&'long $($lock)? [ElementType]>
        }
        impl<'long, ElementType: BitOps> Iterator for $name<'long, ElementType> {
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
                match unsafe { self.try_fold_rword(init, |mut accum,range,word| {
                    let wordp = word as *$ptr_ty ElementType;
                    for bit_pos in range {
                        let bit =  (*wordp).$bit_method(bit_pos);
                        accum = f(accum, bit);
                    }
                    ControlFlow::Continue(accum)
                })} { ControlFlow::Break(value) | ControlFlow::Continue(value) => value }
            }

            // fn position<P:FnMut(Self::Item) -> bool>(&mut self, predicate: P) -> Option<usize>
            // where Self: Sized {

            // }

        }

        impl<'long, ElementType: BitOps> $name<'long,ElementType>{
            /// Biterator from a start pointer, start bit and remaining bits
            pub unsafe fn new(current_pointer:*$ptr_ty ElementType, bit_position:u8, remaining_bits:usize) -> Self {Self {current_pointer, bit_position, remaining_bits, _slicelife:PhantomData} }
            /// Remaining bits to iterate over (self.remaining_bits)
            pub fn remaining_bits(&self) -> usize {self.remaining_bits}
            /// Biterator from a number
            pub fn from_num(s:&'long $($lock)? ElementType) -> Self { unsafe {Self::new(s as *$ptr_ty ElementType,0,ElementType::BITS as usize)}}
            /// Add (or subtract) a amount to remaining_bits, resizing the iterator
            pub unsafe fn uncheked_resize_bits(&mut self, resize_amount:isize) {
                self.remaining_bits=self.remaining_bits.wrapping_add_signed(resize_amount) // Wraps
            }

            // if it breaks i need to know some value and the bit_positon it broke at (B,u8)
            pub unsafe fn try_fold_rword<B,F: FnMut(B, Range<u8>, &'long $($lock)? ElementType) -> ControlFlow<(B,u8), B>,>(&mut self, init: B, mut f: F) -> ControlFlow<B, B> {
                let mut accum = init;
                let words:usize = self.remaining_bits.div_ceil(ElementType::BITS as usize);

                let mut matchf = |accum, bit_range, word| {
                    match f(accum,bit_range,word) {
                        ControlFlow::Continue(next_accum) => accum = next_accum,
                        ControlFlow::Break((break_val,new_bit_position)) => {
                            self.remaining_bits-=(new_bit_position-self.bit_position) as usize; //breaks if new_bit_positon is less than current bit_position or greater than number of bits in a word which shouldnt be possible if the caller properly uses the range
                            self.bit_position=new_bit_position;
                            return ControlFlow::Break(break_val)
                        }
                    }
                };

                let mut ends = |word_start,word_end| {
                    let range = self.bit_position..word_end;
                    let range_length = range.len();
                    matchf(accum,range, unsafe{&$($lock)? *self.current_pointer});
                    self.remaining_bits-=range_length;
                };

                let mut middle = |full_words| {
                    for full_word in 0..full_words {
                        matchf(accum, 0..(ElementType::BITS as u8), unsafe{&$($lock)? *self.current_pointer});
                        self.remaining_bits-=ElementType::BITS as usize;
                        unsafe {self.current_pointer = self.current_pointer.add(1)}
                    }
                    unsafe {self.current_pointer = self.current_pointer.sub(1)}
                };

                match words {
                    0 => {return ControlFlow::Continue(accum)}, // exit
                    1 => {ends(self.bit_position,self.remaining_bits as u8)}, // start
                    2 => {
                            ends(self.bit_position,ElementType::BITS as u8);
                            unsafe {self.current_pointer = self.current_pointer.add(1)};
                            self.bit_position=0;

                            ends(0,self.remaining_bits as u8)
                    }, // start end
                    _ => {
                        ends(self.bit_position,ElementType::BITS as u8);
                        unsafe {self.current_pointer = self.current_pointer.add(1)};
                        self.bit_position=0;

                        middle(words-2);
                        unsafe {self.current_pointer = self.current_pointer.add(1)};
                        self.bit_position=0;

                        ends(0,self.remaining_bits as u8);
                    }  // start middle end
                }
                ControlFlow::Continue(accum)
            }

            pub unsafe fn wordsrangefold<B, F: FnMut(B,Range<u8>, &'long $($lock)? ElementType) -> B>(&mut self,init:B,mut f:F) -> B {
                unsafe { match self.try_fold_rword(init, |accum, range, element| ControlFlow::Continue(f(accum, range, element))) {
                    ControlFlow::Break(value) | ControlFlow::Continue(value) => value
                } }
            }

            pub unsafe fn position_rword<F: FnMut(Range<u8>, &'long $($lock)? ElementType) -> Option<u8> >(&mut self,mut f:F) -> Option<(usize,u8)> {
                let start_ptr = self.current_pointer;
                unsafe { match self.try_fold_rword(None, |_, range,word| {
                    let offset = unsafe {(word as *const ElementType).offset_from(start_ptr) as usize};
                    if let Some(bit_pos) = f(range,word) {
                        ControlFlow::Break(Some((offset,bit_pos)))
                    } else {ControlFlow::Continue(None)}
                }) {ControlFlow::Break(value) | ControlFlow::Continue(value) => value}
                }
            }


            pub fn popcnt(&mut self) -> usize {
                unsafe {self.wordsrangefold(0,|accum, range,word| accum+word.popcnt(&range) as usize)}
            }

            pub fn ctz(&mut self) -> usize {
                unsafe {self.wordsrangefold(0,|accum, range,word| accum+word.ctz(&range) as usize)}
            }
        }

        /// Biterator from anything that can be sliced (collections)
        impl <'long,ElementType: BitOps,S:?Sized+AsRef<[ElementType]>+$($($sp)*)? > From<&'long $($lock)? S> for $name<'long,ElementType> {
            fn from( s:&'long $($lock)? S) -> Self {unsafe {Self::new(s.$to_slice() as *$ptr_ty [ElementType] as *$ptr_ty ElementType,0,s.as_ref().len()*(ElementType::BITS as usize)) }}
        }
    }
}
biterators!(name:Biter,item:bool,bit_method:get_bit,to_slice:as_ref, ptr_ty:const);
biterators!(name:MutBiter,item:MutBitProxy<'long,ElementType>,bit_method:mut_bit,(S:AsMut<[ElementType]>),to_slice:as_mut, ptr_ty:mut, lock:mut);
