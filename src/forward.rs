use crate::{Biter,MutBiter};
use bit_operations::{BitOps,MutBitProxy};
use core::ops::{Range,ControlFlow};
macro_rules! biterators {
    (name:$name:ident, item:$item:ty, bit_method:$bit_method:ident, $((S:$($sp:tt)*),)?to_slice:$to_slice:ident, ptr_ty:$ptr_ty:tt  $(, lock:$lock:tt)? ) => {

        impl<'long, ElementType: BitOps> Iterator for $name<'long, ElementType> {
            type Item = $item;
            fn next(&mut self) -> Option<Self::Item> {
                if self.remaining_bits==0 {return None}
                let bit = unsafe {(*self.start_pointer).$bit_method(self.start_bit) };
                self.remaining_bits-=1;
                self.start_bit+=1;
                if self.start_bit==ElementType::BITS as u8 {
                    self.start_bit=0;
                    unsafe {self.start_pointer = self.start_pointer.add(1)};
                }
                Some(bit)
            }
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
            fn size_hint(&self) -> (usize, Option<usize>) {(self.remaining_bits, Some(self.remaining_bits))}
        }
        impl<'long, ElementType: BitOps> ExactSizeIterator for $name<'long,ElementType> {} //uses size_hint

        impl<'long, ElementType: BitOps> $name<'long,ElementType>{
            /// try_fold on whole words, passes accum,bit_range,word to f, f must return control flow, on controlflow::break, new_accum,bit_pos_break must be returned
            pub unsafe fn try_fold_rword<B,F: FnMut(B, Range<u8>, &'long $($lock)? ElementType) -> ControlFlow<(B,u8), B>,>(&mut self, init: B, mut f: F) -> ControlFlow<B, B> {
                if self.remaining_bits == 0 {return ControlFlow::Continue(init);} //early exit
                let mut accum = init;
                let words:usize = self.words();

                let slefp = self as *mut Self;
                let mut matchf= |accum:B,bit_range:Range<u8>,word:&'long $($lock)? ElementType|{
                    unsafe {match f(accum,bit_range.clone(),word) {
                        ControlFlow::Continue(next_accum) => {
                            (*slefp).remaining_bits-=bit_range.len();
                            return ControlFlow::Continue(next_accum)
                        },
                        ControlFlow::Break((break_val,new_start_bit)) => {
                            (*slefp).remaining_bits-=(new_start_bit-bit_range.start) as usize; //breaks if new_bit_positon is less than current start_bit or greater than number of bits in a word which shouldnt be possible if the caller properly uses the range
                            (*slefp).start_bit=new_start_bit;
                            return ControlFlow::Break(break_val)
                        }
                    }}
                };

                if words>=2 { // start
                    accum = matchf(accum,self.start_bit..ElementType::BITS as u8,unsafe{&$($lock)? *self.start_pointer})?;
                    unsafe {self.start_pointer = self.start_pointer.add(1)};
                    self.start_bit=0;
                }

                for _ in 0..words.saturating_sub(2) { // middle
                    accum = matchf(accum, 0..(ElementType::BITS as u8),unsafe{&$($lock)? *self.start_pointer})?;
                    unsafe {self.start_pointer = self.start_pointer.add(1)}
                }
                // end
                accum = matchf(accum,self.start_bit..(self.end_bit+1),unsafe{&$($lock)? *self.end_pointer})?;
                self.start_bit = self.end_bit;

                ControlFlow::Continue(accum)
            }

            ///takes a inital value and a function that accepts: a accumulator, a bitrange and a word, the function it accepts must return a new accumulator when it runs, when wordsrangefold is finished it will return that accumulator
            pub unsafe fn wordsrangefold<B, F: FnMut(B,Range<u8>, &'long $($lock)? ElementType) -> B>(mut self,init:B,mut f:F) -> B {
                unsafe { match self.try_fold_rword(init, |accum, range, element| ControlFlow::Continue(f(accum, range, element))) {
                    ControlFlow::Break(value) | ControlFlow::Continue(value) => value
                } }
            }
            ///position on whole words, f must return Option<bit_pos>, if some it short-circuits.
            pub unsafe fn position_rword<F: FnMut(Range<u8>, &'long $($lock)? ElementType) -> Option<u8> >(&mut self,mut f:F) -> Option<usize> {
                let obits = self.remaining_bits;
                if unsafe { self.try_fold_rword((), |_, range,word| {
                        if let Some(bit_pos) = f(range,word) { ControlFlow::Break(((),bit_pos))}
                        else {ControlFlow::Continue(())} })}.is_break() {
                    Some(obits-self.remaining_bits)
                } else {None}
            }
            ///find first one in this iterator. consumes iterator
            pub fn first_one(mut self) -> Option<usize> {
               unsafe { self.position_rword(|range,word| {word.first_one(&range)}) }
            }
            ///find first zero in this iterator. consumes iterator
            pub fn first_zero(mut self) -> Option<usize> {
               unsafe { self.position_rword(|range,word| {word.first_zero(&range)}) }
            }
            ///count number of ones in this iterator. consumes iterator
            pub fn popcnt(self) -> usize {
                unsafe {self.wordsrangefold(0,|accum, range,word| accum+word.popcnt(&range) as usize)}
            }
            ///count number of zero in this iterator. consumes iterator
            pub fn ctz(self) -> usize {
                unsafe {self.wordsrangefold(0,|accum, range,word| accum+word.ctz(&range) as usize)}
            }
        }
    }
}
biterators!(name:Biter,item:bool,bit_method:get_bit,to_slice:as_ref, ptr_ty:const);
biterators!(name:MutBiter,item:MutBitProxy<'long,ElementType>,bit_method:mut_bit,(S:AsMut<[ElementType]>),to_slice:as_mut, ptr_ty:mut, lock:mut);
