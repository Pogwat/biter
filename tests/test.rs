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
    let mut array_slice = &mut array[0..4];
    let mutbiter=MutBiter::from(array_slice);
    assert_eq!(mutbiter.remaining_bits(),8*4);
    mutbiter.for_each(|mut bit| *bit=true);
    assert_eq!(array[0],u8::MAX);
    assert_eq!(array[1],u8::MAX);
    assert_eq!(array[2],u8::MAX);
    assert_eq!(array[3],u8::MAX);

    let mutbiter_a = MutBiter::from(&mut array);
    println!("{}",&mutbiter_a.remaining_bits());
    mutbiter_a.for_each(|mut bit| {println!("{}",*bit); assert_eq!(*bit,true);});
    assert_eq!(array.iter().map(|&n| n as usize).sum::<usize>(), 4*(u8::MAX as usize));
}
