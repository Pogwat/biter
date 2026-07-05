#![no_std]
#[doc = include_str!("../README.md")]
use core::marker::PhantomData;
use bit_operations::BitOps;
macro_rules! biterators {
    (name:$name:ident, item:$item:ty, bit_method:$bit_method:ident, ptr_ty:$ptr_ty:tt  $(, lock:$lock:tt)? ) => {
        ///The Bit Iterator
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
                    let bit = unsafe {(*self.current_pointer).$bit_method(self.bit_position as usize) };
                    self.bit_position+=1;
                    self.remaining_bits-=1;
                    if self.bit_position==ElementType::TYPE_BITS as u8 {
                        self.bit_position=0;
                        unsafe {self.current_pointer = self.current_pointer.add(1)};
                    }
                    Some(bit)
                } else {None}
            }
        }
        impl<'short, ElementType: BitOps> $name<'short,ElementType>{
            pub unsafe fn new(current_pointer:*$ptr_ty ElementType, bit_position:u8, remaining_bits:usize) -> Self {Self {current_pointer, bit_position, remaining_bits, _bitlife: PhantomData} }
        }

        impl <'short,ElementType: BitOps > From<&'short $($lock)? [ElementType]> for $name<'short,ElementType> {
            fn from(s:&'short $($lock)? [ElementType]) -> Self {
                unsafe {Self::new(s as *$ptr_ty [ElementType] as *$ptr_ty ElementType,0,s.len()*ElementType::TYPE_BITS)}
            }
        }

        impl <'short,ElementType: BitOps > From<&'short $($lock)? ElementType> for $name<'short,ElementType> {
            fn from(s:&'short $($lock)? ElementType) -> Self {
                unsafe {Self::new(s as *$ptr_ty ElementType,0,ElementType::TYPE_BITS)}
            }
        }
    }
}
biterators!(name:Biter,item:bool,bit_method:get_bit, ptr_ty:const);
biterators!(name:MutBiter,item:MutBitProxy<'short,ElementType>,bit_method:mut_bit, ptr_ty:mut, lock:mut);