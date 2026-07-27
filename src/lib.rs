#![no_std]
#[doc = include_str!("../README.md")]
use core::marker::PhantomData;
use bit_operations::{BitOps,MutBitProxy};
use core::ops::{Range,ControlFlow};
macro_rules! biterators {
    (name:$name:ident, item:$item:ty, bit_method:$bit_method:ident, $((S:$($sp:tt)*),)?to_slice:$to_slice:ident, ptr_ty:$ptr_ty:tt  $(, lock:$lock:tt)? ) => {
        /// A Bit Iterator
        pub struct $name<'long,ElementType> {
            start_pointer: *$ptr_ty ElementType,
            start_bit:u8,
            end_pointer:*$ptr_ty ElementType,
            end_bit:u8,
            remaining_bits: usize,
            _slicelife: PhantomData<&'long $($lock)? [ElementType]>
        }

        macro_rules! fold {($fold_name:ident , $method_to_match:ident)=>{
            fn $fold_name<B, F: FnMut(B, Self::Item) -> B>(mut self, init: B, mut f: F) -> B {
                match unsafe { self.$method_to_match(init, |mut accum,range,word| {
                    let wordp = word as *$ptr_ty ElementType;
                    for bit_pos in range {
                        let bit =  (*wordp).$bit_method(bit_pos);
                        accum = f(accum, bit);
                    }
                    ControlFlow::Continue(accum)
                })} { ControlFlow::Break(value) | ControlFlow::Continue(value) => value }
            }
        }}

        impl<'long, ElementType: BitOps> Iterator for $name<'long, ElementType> {
            type Item = $item;
            fn next(&mut self) -> Option<Self::Item> {
                if self.remaining_bits!=0 {
                    let bit = unsafe {(*self.start_pointer).$bit_method(self.start_bit) };
                    self.remaining_bits-=1;
                    self.start_bit+=1;
                    if self.start_bit==ElementType::BITS as u8 {
                        self.start_bit=0;
                        unsafe {self.start_pointer = self.start_pointer.add(1)};
                    }
                    Some(bit)
                } else {None}
            }
            fold!(fold,try_fold_rword);
            fn size_hint(&self) -> (usize, Option<usize>) {(self.remaining_bits, Some(self.remaining_bits))}
        }
        impl<'long, ElementType: BitOps> ExactSizeIterator for $name<'long,ElementType> {} //uses size_hint

        impl<'long, ElementType: BitOps> DoubleEndedIterator for $name<'long,ElementType> {
            fn next_back(&mut self) -> Option<Self::Item> {
                if self.remaining_bits!=0 {
                        let bit = unsafe {(*self.end_pointer).$bit_method(self.end_bit) };
                        self.remaining_bits-=1;
                        if self.end_bit==0 {
                            self.end_bit=ElementType::BITS as u8; //Invalid
                            unsafe {self.end_pointer = self.end_pointer.sub(1)};
                        }
                        self.end_bit-=1; //Valid
                        Some(bit)
                    } else {None}
                }
               fold!(rfold,rtry_fold_rword);
        }

        impl<'long, ElementType: BitOps> $name<'long,ElementType>{
            /// Biterator from a start pointer, start bit and remaining bits
            pub unsafe fn from_ptr_bitpos_rembits(start_pointer:*$ptr_ty ElementType,start_bit:u8,remaining_bits:usize) -> Self {
                unsafe {
                    let bits = (remaining_bits+start_bit as usize).saturating_sub(1);
                    let end_pointer = start_pointer.add(bits/ElementType::BITS as usize);
                    let end_bit = (bits%ElementType::BITS as usize) as u8;
                    Self {start_pointer,start_bit,end_pointer,end_bit,remaining_bits,_slicelife:PhantomData}
                }
                }
            /// Remaining bits to iterate over (self.remaining_bits)
            pub fn remaining_bits(&self) -> usize {self.remaining_bits}
            /// Biterator from start pointer, start_bit, end pointer , end_bit
            pub unsafe fn new(start_pointer:*$ptr_ty ElementType, start_bit:u8, end_pointer:*$ptr_ty ElementType, end_bit:u8)-> Self {
                let remaining_bits = unsafe {(end_pointer.offset_from(start_pointer) as usize)*ElementType::BITS as usize +end_bit as usize -start_bit as usize+1};
                Self {start_pointer,start_bit,end_pointer,end_bit,remaining_bits,_slicelife:PhantomData}
            }
            /// Biterator from a number
            pub fn from_num(s:&'long $($lock)? ElementType) -> Self {
                let sptr = s as *$ptr_ty ElementType;
                unsafe {Self::new(sptr,0,sptr,ElementType::BITS as u8 -1)}
            }
            /// Add (or subtract) a amount to remaining_bits, resizing the iterator
            pub unsafe fn uncheked_resize_bits(&mut self, resize_amount:isize) {
                self.remaining_bits=self.remaining_bits.wrapping_add_signed(resize_amount) // Wraps
            }
            /// try_fold on whole words, passes accum,bit_range,word to f, f must return control flow, on controlflow::break, new_accum,bit_pos_break must be returned
            pub unsafe fn try_fold_rword<B,F: FnMut(B, Range<u8>, &'long $($lock)? ElementType) -> ControlFlow<(B,u8), B>,>(&mut self, init: B, mut f: F) -> ControlFlow<B, B> {
                if self.remaining_bits == 0 {return ControlFlow::Continue(init);} //early exit
                let mut accum = init;
                let words:usize = (self.remaining_bits+self.start_bit as usize).div_ceil(ElementType::BITS as usize); //if remaining_bits is 0 this is wrong: (0+4).div_ceil()==1 even though no bits remain

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

            /// reverse try fold, passes accum,bitrange,word for each iteration to func, on break f must return new accum, break_bit_position
            pub unsafe fn rtry_fold_rword<B,F: FnMut(B, Range<u8>, &'long $($lock)? ElementType) -> ControlFlow<(B,u8), B>,>(&mut self, init: B, mut f: F) -> ControlFlow<B, B> {
                if self.remaining_bits == 0 {return ControlFlow::Continue(init);} //early exit
                let mut accum = init;
                let words:usize = (self.remaining_bits+self.start_bit as usize).div_ceil(ElementType::BITS as usize); //if remaining_bits is 0 this is wrong: (0+4).div_ceil()==1 even though no bits remain

                let slefp = self as *mut Self;
                let mut matchf= |accum:B,bit_range:Range<u8>,word:&'long $($lock)? ElementType|{
                    unsafe {match f(accum,bit_range.clone(),word) {
                        ControlFlow::Continue(next_accum) => {
                            (*slefp).remaining_bits-=bit_range.len();
                            return ControlFlow::Continue(next_accum)
                        },
                        ControlFlow::Break((break_val,new_start_bit)) => {
                            (*slefp).remaining_bits-=(bit_range.end-new_start_bit) as usize; //breaks if new_bit_positon is less than current start_bit or greater than number of bits in a word which shouldnt be possible if the caller properly uses the range
                            (*slefp).end_bit=new_start_bit;
                            return ControlFlow::Break(break_val)
                        }
                    }}
                };

                if words>=2 { // start
                    accum = matchf(accum,0..(self.end_bit+1),unsafe{&$($lock)? *self.end_pointer})?;
                    unsafe {self.end_pointer = self.end_pointer.sub(1)};
                    self.end_bit=ElementType::BITS as u8 -1;
                }

                for _ in 0..words.saturating_sub(2) { // middle
                    accum = matchf(accum, 0..(ElementType::BITS as u8),unsafe{&$($lock)? *self.end_pointer})?;
                    unsafe {self.end_pointer = self.end_pointer.sub(1)};
                }
                // end
                accum = matchf(accum,self.start_bit..(self.end_bit+1),unsafe{&$($lock)? *self.end_pointer})?;
                self.end_bit = self.start_bit;

                ControlFlow::Continue(accum)
            }
            ///reverse position on whole words, f must return Option<bit_pos>, if some it short-circuits.
            pub unsafe fn rposition_rword<F: FnMut(Range<u8>, &'long $($lock)? ElementType) -> Option<u8> >(&mut self,mut f:F) -> Option<usize> {
                if unsafe { self.rtry_fold_rword((), |_, range,word| {
                        if let Some(bit_pos) = f(range,word) { ControlFlow::Break(((),bit_pos))}
                        else {ControlFlow::Continue(())} })}.is_break() {
                    Some(self.remaining_bits)
                } else {None}
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
            ///find last one in this iterator. consumes iterator
            pub fn last_one(mut self) -> Option<usize> {
                unsafe { self.rposition_rword(|range,word| {word.last_one(&range)}) }
            }
            ///find last zero in this iterator. consumes iterator
            pub fn last_zero(mut self) -> Option<usize> {
                unsafe { self.rposition_rword(|range,word| {word.last_zero(&range)}) }
            }
            ///get a bit in this iterator, equivlent to nth() but dosent mutate iterator, no bounds check
            pub unsafe fn get_uncheked(& $($lock)? self, position:usize) -> <Self as Iterator>::Item {
                let real_position = position+self.start_bit as usize;
                let bit_in_element = (real_position%ElementType::BITS as usize) as u8; //equivlent to real_position&(ElementType::BITS-1)
                let ptr_offset = real_position/ElementType::BITS as usize; //equivlent to real_position>>ElementType::TYPE_BITS
                unsafe {(*(self.start_pointer.add(ptr_offset))).$bit_method(bit_in_element) }
            }
            ///get a bit in this iterator, equivlent to nth() but dosent mutate iterator
            pub fn get(& $($lock)? self, position:usize) -> <Self as Iterator>::Item {
                assert!(position<=self.remaining_bits, "position {} is greter then iterator len {}",position,self.remaining_bits);
                unsafe {self.get_uncheked(position)}
            }
        }

        /// Biterator from anything that can be sliced (collections)
        impl <'long,ElementType: BitOps,S:?Sized+AsRef<[ElementType]>+$($($sp)*)? > From<&'long $($lock)? S> for $name<'long,ElementType> {
            fn from( s:&'long $($lock)? S) -> Self {
                unsafe {
                    let ptr_offset=s.as_ref().len().saturating_sub(1);
                    let start_pointer=s.$to_slice() as *$ptr_ty [ElementType] as *$ptr_ty ElementType;
                    Self::new(start_pointer,0,start_pointer.add(ptr_offset),ElementType::BITS as u8 -1)
                }
            }
        }
    }
}
biterators!(name:Biter,item:bool,bit_method:get_bit,to_slice:as_ref, ptr_ty:const);
biterators!(name:MutBiter,item:MutBitProxy<'long,ElementType>,bit_method:mut_bit,(S:AsMut<[ElementType]>),to_slice:as_mut, ptr_ty:mut, lock:mut);
