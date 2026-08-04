#![no_std]
#[doc = include_str!("../README.md")]
use core::marker::PhantomData;
use bit_operations::BitOps;
mod backward;
mod forward;
macro_rules! biterators {
    (name:$name:ident, item:$item:ty, bit_method:$bit_method:ident, $((S:$($sp:tt)*),)?to_slice:$to_slice:ident, ptr_ty:$ptr_ty:tt  $(, lock:$lock:tt)? ) => {
        /// A Bit Iterator
        pub struct $name<'long,ElementType> {
            ///the pointer this iterator starts at inclusive
            start_pointer: *$ptr_ty ElementType,
            ///the bit this iterator starts at inclusive
            start_bit:u8,
            ///the pointer this iterator ends at inclusive
            end_pointer:*$ptr_ty ElementType,
            ///the bit this iterator ends at exclsuive
            end_bit:u8,
            ///how many bits are left in this iterator (len)
            remaining_bits: usize,
            _slicelife: PhantomData<&'long $($lock)? [ElementType]>
        }

        impl<'long, ElementType: BitOps> $name<'long,ElementType>{
            /// full and partial words to process assumes end_pointer is greater than or eaqul to start_pointer
            pub fn words(&self) -> usize {unsafe{self.end_pointer.offset_from_unsigned(self.start_pointer) + (self.remaining_bits!=0) as usize}}
            /// Biterator from a start pointer, start bit and remaining bits
            pub unsafe fn from_ptr_bitpos_rembits(start_pointer:*$ptr_ty ElementType,start_bit:u8,remaining_bits:usize) -> Self {
                unsafe {
                    let bits = start_bit as usize + remaining_bits - (remaining_bits!=0) as usize;
                    let end_bit = (bits&(ElementType::BITS as usize-1)) as u8 + (remaining_bits!=0) as u8;
                    let end_pointer = start_pointer.add(bits/ElementType::BITS as usize);
                    Self {start_pointer,start_bit,end_pointer,end_bit,remaining_bits,_slicelife:PhantomData}
                }
            }
            /// Remaining bits to iterate over (self.remaining_bits)
            pub fn remaining_bits(&self) -> usize { self.remaining_bits}
            /// dynamically calculate Remaining bits to iterate over using start/end pointers and bits
            pub fn dyn_remaining_bits(&self) -> usize {
                let ptr_byte_offset = unsafe {self.end_pointer.byte_offset_from_unsigned(self.start_pointer)};
                ptr_byte_offset*8+self.end_bit as usize - self.start_bit as usize
            }
            /// Biterator from start pointer, start_bit, end pointer , end_bit
            pub unsafe fn new(start_pointer:*$ptr_ty ElementType, start_bit:u8, end_pointer:*$ptr_ty ElementType, end_bit:u8)-> Self {
                let mut self_missing_remaining_bits = Self {start_pointer,start_bit,end_pointer,end_bit,remaining_bits:0,_slicelife:PhantomData};
                self_missing_remaining_bits.remaining_bits = self_missing_remaining_bits.dyn_remaining_bits();
                self_missing_remaining_bits //has remaining_bits now
            }
            /// Biterator from a number
            pub fn from_num(s:&'long $($lock)? ElementType) -> Self {
                let sptr = s as *$ptr_ty ElementType;
                unsafe {Self::new(sptr,0,sptr,ElementType::BITS as u8)}
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
                assert!(position<self.remaining_bits, "position {} is greter then or eaqul to iterator len {}",position,self.remaining_bits);
                unsafe {self.get_uncheked(position)}
            }
        }
        /// Biterator from anything that can be sliced (collections)
        impl <'long,ElementType: BitOps,S:?Sized+AsRef<[ElementType]>+$($($sp)*)? > From<&'long $($lock)? S> for $name<'long,ElementType> {
            fn from( s:&'long $($lock)? S) -> Self {
                unsafe {
                    let ptr_offset=s.as_ref().len().saturating_sub(1);
                    let start_pointer=s.$to_slice() as *$ptr_ty [ElementType] as *$ptr_ty ElementType;
                    Self::new(start_pointer,0,start_pointer.add(ptr_offset),ElementType::BITS as u8)
                }
            }
        }
    }
}
biterators!(name:Biter,item:bool,bit_method:get_bit,to_slice:as_ref, ptr_ty:const);
biterators!(name:MutBiter,item:MutBitProxy<'long,ElementType>,bit_method:mut_bit,(S:AsMut<[ElementType]>),to_slice:as_mut, ptr_ty:mut, lock:mut);
