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

find first zero or one in a collection
```rust
use biter::Biter;
let mut array: [u8;4] = [0,0,0,0];
array[2] = 2; //2*8+1
assert_eq!(Biter::from(&array).first_one(),Some(2*8+1));
assert_eq!(Biter::from(&array).first_zero(),Some(0));
```

find total number of set or unset bits in a collection
```rust
use biter::Biter;
let mut array: [u8;4] = [0,0,0,0];
let mut biter = Biter::from(&array);
assert_eq!(biter.popcnt(),0);
array[2] = u8::MAX;
assert_eq!(Biter::from(&array).ctz(), 3*8);
assert_eq!(Biter::from(&array).popcnt(),8);
```
