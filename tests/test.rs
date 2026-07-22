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
    let biter_ranged = unsafe {Biter::new(&aarray[0] as *const u8, 5, aarray.len()*u8::BITS as usize- 5-2*8+3)};
    let set_bits = biter_ranged.fold(0, |accum,bit| {accum+bit as usize});
    assert_eq!(set_bits,8-5+3);
}

#[test]
fn counters() {
  let mut array: [u8;4] = [0,0,0,0];
  let mut biter = Biter::from(&array);
  assert_eq!(biter.popcnt(),0);
  array[2] = u8::MAX;
  assert_eq!(Biter::from(&array).ctz(), 3*8);
  assert_eq!(Biter::from(&array).popcnt(),8);
}

#[test]
fn firstlast() {
    let mut array: [u8;4] = [0,0,0,0];
    array[2] = 1>>5; //6+2*8
    assert_eq!(Biter::from(&array).first_one(),Some(6+2*8));
}
