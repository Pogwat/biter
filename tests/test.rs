use biter::{Biter,MutBiter};

#[test]
fn biters_num() {
    let mut num:u8 = 0b10110111;
    Biter::from_num(&num).enumerate().for_each(|(idx,bit)| {
        if idx==0 {assert_eq!(bit,true)}
        if idx==1 {assert_eq!(bit,true)}
        if idx==2 {assert_eq!(bit,true)}
        if idx==3 {assert_eq!(bit,false)}
        if idx==4 {assert_eq!(bit,true)}
        if idx==5 {assert_eq!(bit,true)}
        if idx==6 {assert_eq!(bit,false)}
        if idx==7 {assert_eq!(bit,true)}
    });
    MutBiter::from_num(&mut num).enumerate().for_each(|(idx,bit)| {
        if idx==0 {assert_eq!(*bit,true)}
        if idx==1 {assert_eq!(*bit,true)}
        if idx==2 {assert_eq!(*bit,true)}
        if idx==3 {assert_eq!(*bit,false)}
        if idx==4 {assert_eq!(*bit,true)}
        if idx==5 {assert_eq!(*bit,true)}
        if idx==6 {assert_eq!(*bit,false)}
        if idx==7 {assert_eq!(*bit,true)}
    });
}

#[test]
fn biters_slices() {
    let mut array: [u8;4] = [0,0,0,0];
    let array_slice = &mut array[0..4];
    let mutbiter=MutBiter::from(array_slice);
    assert_eq!(mutbiter.remaining_bits(),8*4);
    mutbiter.for_each(|mut bit| *bit=true);
    assert_eq!(array[0],u8::MAX);
    assert_eq!(array[1],u8::MAX);
    assert_eq!(array[2],u8::MAX);
    assert_eq!(array[3],u8::MAX);

    let mutbiter_a = MutBiter::from(&mut array);
    println!("{}",&mutbiter_a.remaining_bits());
    mutbiter_a.for_each(|bit| {println!("{}",*bit); assert_eq!(*bit,true);});
    assert_eq!(array.iter().map(|&n| n as usize).sum::<usize>(), 4*(u8::MAX as usize));

    let biter = Biter::from(&array);
    let set_bits:usize = biter.fold(0, |accum,bit| {accum+bit as usize});
    assert_eq!(set_bits,4*8);

    let aarray:[u8;5] = [!0,0,0,!0,0];
    let biter_ranged = unsafe {Biter::from_ptr_bitpos_rembits(&aarray[0] as *const u8, 5, aarray.len()*u8::BITS as usize- 5-2*8+3)};
    let set_bits = biter_ranged.fold(0, |accum,bit| {accum+bit as usize});
    assert_eq!(set_bits,8-5+3);
}

#[test]
fn counters() {
  let mut array: [u8;4] = [0,0,0,0];
  let biter = Biter::from(&array);
  assert_eq!(biter.popcnt(),0);
  array[2] = u8::MAX;
  assert_eq!(Biter::from(&array).ctz(), 3*8);
  assert_eq!(Biter::from(&array).popcnt(),8);
}

#[test]
fn firstlast() {
    let mut array: [u8;4] = [0,0,0,0];
    array[2] = 2; //6+2*8
    assert_eq!(Biter::from(&array).first_one(),Some(2*8+1));
    assert_eq!(Biter::from(&array).first_zero(),Some(0));
}

#[test]
fn len() {
    let array: [u8;4] = [0,0,0,0];
    assert_eq!(Biter::from(&array).len(), 4*8);
}

#[test]
fn back() {
    let array: [u8;4] = [0,0,0,0b10100000];
    let mut biter = Biter::from(&array);
    assert_eq!(biter.next_back(), Some(true));
    assert_eq!(biter.next_back(), Some(false));
    assert_eq!(biter.next_back(), Some(true));
}
#[test]
fn last() {
    let mut array: [u8;4] = [0,0,0,0b10100000];
    let biter = Biter::from(&array);
    assert_eq!(biter.last_one(), Some(4*8-1));
    array[3]=0;
    array[2]=0b00100000;
    let biter = Biter::from(&array);
    assert_eq!(biter.last_one(), Some(8*2-1  +  6  ));
    let biter = Biter::from(&array);
    assert_eq!(biter.last_zero(), Some(4*8-1));
    let mut array:[u8;7] = [!0,!0,0,!0,0,!0,!0];
    let biter = Biter::from(&array);
    assert_eq!(biter.last_zero(), Some(4*8+7));
    let biter = Biter::from(&array);
    assert_eq!(biter.last_one(), Some(7*8-1));
    array[4]= !0;
    let biter = Biter::from(&array);
    assert_eq!(biter.last_zero(), Some(3*8-1));
    let biter = Biter::from(&array);
    assert_eq!(biter.last_one(), Some(7*8-1));
    array[6]=0;
    let biter = Biter::from(&array);
    assert_eq!(biter.last_one(), Some(6*8-1));

    let array:[u8;7] = [!0,!0,0,!0,0,2,!0];
    let biter = Biter::from(&array);
    let mut set_bits =0;
    biter.rev().for_each(|bit| set_bits+=bit as usize);
    assert_eq!(set_bits,4*8+1); // failed: lhs:32, rhs:33
}

#[test]
fn words() {
    let array: [u8;4] = [0,0,0,0b10100000];
    let biter = Biter::from(&array);
    assert_eq!(biter.words(),4);
    let array: [u32;4] = [0,0,0,0b10100000];
    let biter = Biter::from(&array);
    assert_eq!(biter.words(),4);
}

#[test]
fn from_remaining_bits() { 
    let mut array: [u8;4] = [0,0,0,0b10100000];
    let biter = unsafe {Biter::from_ptr_bitpos_rembits(&mut array as *mut u8,0,4*8)};
    assert_eq!(biter.words(),4);
    assert_eq!(biter.remaining_bits(), 4*8);
    assert_eq!(biter.get(4*8-1),true);
    assert_eq!(biter.get(4*8-2),false);
}
