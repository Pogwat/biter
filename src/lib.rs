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
            bit_position:u8,
            end_pointer:*$ptr_ty ElementType,
            end_bit:u8,
            remaining_bits: usize,
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

            fn size_hint(&self) -> (usize, Option<usize>) {(self.remaining_bits, Some(self.remaining_bits))}
        }
        impl<'long, ElementType: BitOps> ExactSizeIterator for $name<'long,ElementType> {} //uses size_hint

        impl<'long, ElementType: BitOps> DoubleEndedIterator for $name<'long,ElementType> {
            fn next_back(&mut self) -> Option<Self::Item> {
                if self.remaining_bits!=0 {
                        let bit = unsafe {(*self.end_pointer).$bit_method(self.end_bit) };
                        if self.end_bit==0 {
                            self.end_bit=(ElementType::BITS) as u8; //Invalid
                            unsafe {self.end_pointer = self.end_pointer.sub(1)};
                        }
                        self.end_bit-=1; //Now valid
                        self.remaining_bits-=1;
                        Some(bit)
                    } else {None}
                }
                //int + func . provide accum and bit to func
                fn rfold<B, F: FnMut(B, Self::Item) -> B>(mut self, init: B, mut f: F) -> B {
                    match unsafe { self.rtry_fold_rword(init, |mut accum,range,word| {
                        let wordp = word as *$ptr_ty ElementType;
                        for bit_pos in range {
                            let bit =  (*wordp).$bit_method(bit_pos);
                            accum = f(accum, bit);
                        }
                        ControlFlow::Continue(accum)
                    })} { ControlFlow::Break(value) | ControlFlow::Continue(value) => value }
                }
        }

        impl<'long, ElementType: BitOps> $name<'long,ElementType>{
            /// Biterator from a start pointer, start bit and remaining bits
            pub unsafe fn from_ptr_bitpos_rembits(
                current_pointer:*$ptr_ty ElementType,
                bit_position:u8,
                remaining_bits:usize
            ) -> Self {
                unsafe {
                    let bits = (remaining_bits+bit_position as usize).saturating_sub(1);
                    Self {
                        current_pointer,
                        bit_position,
                        end_pointer:current_pointer.add(bits/ElementType::BITS as usize),
                        end_bit: (bits%ElementType::BITS as usize) as u8, //0..ElementType::BITS
                        remaining_bits,
                        _slicelife:PhantomData}}
                }
            /// Remaining bits to iterate over (self.remaining_bits)
            pub fn remaining_bits(&self) -> usize {self.remaining_bits}
            /// Biterator from a number
            pub fn from_num(s:&'long $($lock)? ElementType) -> Self {
                Self {
                    current_pointer: s as *$ptr_ty ElementType,
                    bit_position:0,
                    end_pointer: s as *$ptr_ty ElementType,
                    end_bit: (ElementType::BITS-1) as u8,
                    remaining_bits: ElementType::BITS as usize,
                    _slicelife:PhantomData
                }}
            /// Add (or subtract) a amount to remaining_bits, resizing the iterator
            pub unsafe fn uncheked_resize_bits(&mut self, resize_amount:isize) {
                self.remaining_bits=self.remaining_bits.wrapping_add_signed(resize_amount) // Wraps
            }

            /// takes a function that accepts a accumulator, bitrange and word that must return a controlflow::continue(accumulator) or controlflow::break(accumulator, bit_position), try_fold_rword will return this accumulator on break or after the iterator is fully used up
            pub unsafe fn try_fold_rword<B,F: FnMut(B, Range<u8>, &'long $($lock)? ElementType) -> ControlFlow<(B,u8), B>,>(&mut self, init: B, mut f: F) -> ControlFlow<B, B> {
                if self.remaining_bits == 0 {return ControlFlow::Continue(init);} //early exit
                let mut accum = init;
                let words:usize = (self.remaining_bits+self.bit_position as usize).div_ceil(ElementType::BITS as usize); //if remaining_bits is 0 this is wrong: (0+4).div_ceil()==1 even though no bits remain

                macro_rules! matchf {
                    ($accum:ident, $bit_range:expr, $word:expr) => {
                        {
                            match f($accum,$bit_range,$word) {
                                ControlFlow::Continue(next_accum) => {
                                    let range_length = $bit_range.len();
                                    $accum = next_accum;
                                    self.remaining_bits-=range_length;
                                },
                                ControlFlow::Break((break_val,new_bit_position)) => {
                                    self.remaining_bits-=(new_bit_position-$bit_range.start) as usize; //breaks if new_bit_positon is less than current bit_position or greater than number of bits in a word which shouldnt be possible if the caller properly uses the range
                                    self.bit_position=new_bit_position;
                                    return ControlFlow::Break(break_val)
                                }
                            }
                        }
                    }
                }

                if words>=2 { // start
                    matchf!(accum,self.bit_position..ElementType::BITS as u8,unsafe{&$($lock)? *self.current_pointer});
                    unsafe {self.current_pointer = self.current_pointer.add(1)};
                    self.bit_position=0;
                }

                for _ in 0..words.saturating_sub(2) { // middle
                    matchf!(accum, 0..(ElementType::BITS as u8),unsafe{&$($lock)? *self.current_pointer});
                    unsafe {self.current_pointer = self.current_pointer.add(1)}
                }
                // end
                matchf!(accum,self.bit_position..(self.end_bit+1),unsafe{&$($lock)? *self.end_pointer});
                self.bit_position = self.end_bit;

                ControlFlow::Continue(accum)
            }

            /// takes a function that accepts a accumulator, bitrange and word that must return a controlflow::continue(accumulator) or controlflow::break(accumulator, bit_position), try_fold_rword will return this accumulator on break or after the iterator is fully used up
            pub unsafe fn rtry_fold_rword<B,F: FnMut(B, Range<u8>, &'long $($lock)? ElementType) -> ControlFlow<(B,u8), B>,>(&mut self, init: B, mut f: F) -> ControlFlow<B, B> {
                if self.remaining_bits == 0 {return ControlFlow::Continue(init);} //early exit
                let mut accum = init;
                let words:usize = (self.remaining_bits+self.bit_position as usize).div_ceil(ElementType::BITS as usize); //if remaining_bits is 0 this is wrong: (0+4).div_ceil()==1 even though no bits remain

                macro_rules! matchf {
                    ($accum:ident, $bit_range:expr) => {
                        {
                            match f($accum,$bit_range,unsafe{&$($lock)? *self.end_pointer}) {
                                ControlFlow::Continue(next_accum) => {
                                    let range_length = $bit_range.len();
                                    $accum = next_accum;
                                    self.remaining_bits-=range_length;
                                },
                                ControlFlow::Break((break_val,new_bit_position)) => {
                                    self.remaining_bits-=($bit_range.end - new_bit_position) as usize; //breaks if new_bit_positon is less than current bit_position or greater than number of bits in a word which shouldnt be possible if the caller properly uses the range
                                    self.end_bit=new_bit_position;
                                    return ControlFlow::Break(break_val)
                                }
                            }
                        }
                    }
                }

                if words>=2 { // start
                    matchf!(accum,0..(self.end_bit+1));
                    unsafe {self.end_pointer = self.end_pointer.sub(1)};
                    self.end_bit=ElementType::BITS as u8 -1;
                }

                for _ in 0..words.saturating_sub(2) { // middle
                    matchf!(accum, 0..(ElementType::BITS as u8));
                    unsafe {self.end_pointer = self.end_pointer.sub(1)};
                }
                // end
                matchf!(accum,self.bit_position..(self.end_bit+1));
                self.end_bit = self.bit_position;

                ControlFlow::Continue(accum)
            }
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
            ///takes a function that accepts and bitrange and word, this function must return a Option containing a bit position, when it does this position_rword will short circuit and return the bit positon in the bit iterator that function stopped at
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
                let real_position = position+self.bit_position as usize;
                let bit_in_element = (real_position%ElementType::BITS as usize) as u8; //equivlent to real_position&(ElementType::BITS-1)
                let ptr_offset = real_position/ElementType::BITS as usize; //equivlent to real_position>>ElementType::TYPE_BITS
                unsafe {(*(self.current_pointer.add(ptr_offset))).$bit_method(bit_in_element) }
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
                    let current_pointer=s.$to_slice() as *$ptr_ty [ElementType] as *$ptr_ty ElementType;
                    Self {
                        current_pointer,
                        bit_position:0,
                        end_pointer: current_pointer.add(ptr_offset),
                        end_bit: (ElementType::BITS-1) as u8,
                        remaining_bits: s.as_ref().len()*ElementType::BITS as usize,
                        _slicelife:PhantomData
                    }}
        }
        }
    }
}
biterators!(name:Biter,item:bool,bit_method:get_bit,to_slice:as_ref, ptr_ty:const);
biterators!(name:MutBiter,item:MutBitProxy<'long,ElementType>,bit_method:mut_bit,(S:AsMut<[ElementType]>),to_slice:as_mut, ptr_ty:mut, lock:mut);
