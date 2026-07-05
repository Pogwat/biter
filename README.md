# Biter
Bit Iteration as fast as possible
turn anything that can be sliced into a bit iterator

## Examples

immutably iterating over a collection
```rust
use biter::Biter;
let array: [u16;2] =[1,2];
let biter = Biter::from(&array);
let mut set_bits = 0;
biter.for_each(|bit| set_bits+=bit as usize);
assert_eq!(set_bits,2);
```

mutably iterating over a collection
```rust
use biter::MutBiter;
let mut array: [u16;2] =[1,2];
let biter = MutBiter::from(&mut array);
biter.for_each(|mut bit| *bit=true);
assert_eq!(array[0] as usize +array[1] as usize ,u16::MAX as usize*2);
```

mutably or immutably iterating over a number
```rust
use biter::{Biter,MutBiter};
let mut num: u8 =8;
let biter = MutBiter::from_num(&mut num); //Mutable
biter.for_each(|mut bit| *bit=true);
assert_eq!(num,u8::MAX);

let biter = Biter::from_num(&num); //Immutable
biter.for_each(|bit|assert_eq!(bit,true))
```