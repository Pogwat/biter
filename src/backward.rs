use crate::{MutBiter,Biter};
use bit_operations::BitOps;
use core::ops::{Range,ControlFlow};
macro_rules! biterators {
    (name:$name:ident, item:$item:ty,bit_method:$bit_method:ident,$((S:$($sp:tt)*),)?to_slice:$to_slice:ident,ptr_ty:$ptr_ty:tt$(, lock:$lock:tt)? ) => {
        impl<'long, ElementType: BitOps> DoubleEndedIterator for $name<'long,ElementType> {
            fn next_back(&mut self) -> Option<Self::Item> {
                if self.remaining_bits==0 {return None}
                self.end_bit-=1; //end_bit is exclusive so we must sub to make it valid
                let bit = unsafe {(*self.end_pointer).$bit_method(self.end_bit) };
                self.remaining_bits-=1;
                if self.end_bit==0 {
                    self.end_bit=ElementType::BITS as u8;
                    unsafe {self.end_pointer = self.end_pointer.sub(1)};
                }
                Some(bit)
            }
            fn rfold<B, F: FnMut(B, Self::Item) -> B>(mut self, init: B, mut f: F) -> B {
                unsafe { self.rtry_fold_rword(init, |mut accum,range,word| {
                    let wordp = word as *$ptr_ty ElementType;
                    for bit_pos in range {
                        let bit =  (*wordp).$bit_method(bit_pos);
                        accum = f(accum, bit);
                    }
                    ControlFlow::Continue(accum)
                })}.continue_value().unwrap()
            }
        }

        impl<'long, ElementType: BitOps> $name<'long,ElementType> {
            /// reverse try fold, passes accum,bitrange,word for each iteration to func, on break f must return new accum, break_bit_position
            //NOTE: reamining_bits is only updatetd when the funciton exists meaning it is innacurtate during the runtime of this func
            pub unsafe fn rtry_fold_rword<B,F: FnMut(B, Range<u8>, &'long $($lock)? ElementType) -> ControlFlow<(B,u8), B>,>(&mut self, init: B, mut f: F) -> ControlFlow<B, B> {
                if self.remaining_bits == 0 {return ControlFlow::Continue(init);} //early exit
                let mut accum = init;
                let words = self.words();

                let slefp = self as *mut Self;
                let mut matchf= |accum:B,bit_range:Range<u8>,word:&'long $($lock)? ElementType|{
                    unsafe {match f(accum,bit_range,word) {
                        ControlFlow::Continue(next_accum) => {return ControlFlow::Continue(next_accum)},
                        ControlFlow::Break((break_val,new_start_bit)) => {
                            (*slefp).end_bit=new_start_bit; //breaks if new_bit_positon is less than current start_bit or greater than number of bits in a word which shouldnt be possible if the caller properly uses the range
                            (*slefp).remaining_bits = (*slefp).dyn_remaining_bits();
                            return ControlFlow::Break(break_val)
                        }
                    }}
                };

                if words>=2 { // start
                    accum = matchf(accum,0..self.end_bit,unsafe{&$($lock)? *self.end_pointer})?;
                    unsafe {self.end_pointer = self.end_pointer.sub(1)};
                    self.end_bit=ElementType::BITS as u8;

                    for _ in 0..words-2 { // middle
                        accum = matchf(accum, 0..(ElementType::BITS as u8),unsafe{&$($lock)? *self.end_pointer})?;
                        unsafe {self.end_pointer = self.end_pointer.sub(1)};
                    }
                }
                // end
                accum = matchf(accum,self.start_bit..self.end_bit,unsafe{&$($lock)? *self.end_pointer})?;
                self.end_bit = self.start_bit;

                self.remaining_bits = self.dyn_remaining_bits();
                ControlFlow::Continue(accum)
            }
            ///reverse position on whole words, f must return Option<bit_pos>, if some it short-circuits.
            pub unsafe fn rposition_rword<F: FnMut(Range<u8>, &'long $($lock)? ElementType) -> Option<u8> >(&mut self,mut f:F) -> Option<usize> {
                unsafe { self.rtry_fold_rword((), |_, range,word|
                        match f(range,word) {
                            Some(bit_pos) => {ControlFlow::Break(((),bit_pos))}
                            None => {ControlFlow::Continue(())}
                        }
                    )}.is_break().then(|| self.remaining_bits)
            }
            ///find last one in this iterator. consumes iterator
            pub fn last_one(mut self) -> Option<usize> {
                unsafe { self.rposition_rword(|range,word| {word.last_one(&range)}) }
            }
            ///find last zero in this iterator. consumes iterator
            pub fn last_zero(mut self) -> Option<usize> {
                unsafe { self.rposition_rword(|range,word| {word.last_zero(&range)}) }
            }
        }
    }
}
biterators!(name:Biter,item:bool,bit_method:get_bit,to_slice:as_ref, ptr_ty:const);
biterators!(name:MutBiter,item:MutBitProxy<'long,ElementType>,bit_method:mut_bit,(S:AsMut<[ElementType]>),to_slice:as_mut, ptr_ty:mut, lock:mut);
