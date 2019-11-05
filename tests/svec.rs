use stacked::{SVec, SVec4};

use kerr::KErr;

use std::mem::size_of;

#[test]
fn svec1() {
    let vec = SVec4::<i32>::new();
    let ai = vec.push(0).unwrap();
    let bi = vec.push(1).unwrap();
    eprintln!("{} {}",vec.get(ai),vec.get(bi));
    vec.push(2).unwrap();
    vec.push(3).unwrap();

    assert_eq!(vec.push(4), Err(KErr::new("overflow")));
}



#[derive(PartialEq)]
struct Dropper(i32);
impl Drop for Dropper {
    fn drop(&mut self) {
        eprintln!("in Dropper.drop: {}", self.0);
    }
}
impl Default for Dropper {
    fn default() -> Self { Self(0) }
}

#[test]
fn svec2() {
    eprintln!("I expect to see: START 3 2 1 END");
    let vec = SVec4::<Dropper>::new();
    assert_eq!(vec.len(),0);
    vec.push(Dropper(1)).unwrap();
    vec.push(Dropper(2)).unwrap();
    vec.push(Dropper(3)).unwrap();
}

#[test]
fn optionlayout() {
    eprintln!("i32 size: {},  Option<i32> size: {}", size_of::<i32>(), size_of::<Option<i32>>());
}

#[test]
fn mutation() {
    let mut vec = SVec4::<i32>::new();
    vec.push(1).unwrap();
    vec.push(2).unwrap();
    vec.push(3).unwrap();
    assert_eq!(format!("{}",vec), "SVec4[ 1, 2, 3 ]");

    assert_eq!(vec.pop(), 3);
    assert_eq!(format!("{}",vec), "SVec4[ 1, 2 ]");
    vec.push(4).unwrap();
    assert_eq!(format!("{}",vec), "SVec4[ 1, 2, 4 ]");
    
    vec.set(0,5);
    assert_eq!(format!("{}",vec), "SVec4[ 5, 2, 4 ]");

    vec.insert(1,6);
    assert_eq!(format!("{}",vec), "SVec4[ 5, 6, 2, 4 ]");

    vec.remove(1);
    assert_eq!(format!("{}",vec), "SVec4[ 5, 2, 4 ]");

    let x = vec.get(1);   // Get shared access.
    assert_eq!(x, &2);

    vec.push(7).unwrap(); // Append is still ok even with 'x' around.

    vec.set(2,8);         // Mutation is not allowed while 'x' exists.
    vec.remove(2);
    vec.pop();
    vec.pop();
    assert_eq!(format!("{}",vec), "SVec4[ 5 ]");

    // assert_eq!(x, &2); // Uncomment to test borrow check.
}

