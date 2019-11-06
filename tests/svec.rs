use stacked::{SVec, SVec4, SVec16};

use kerr::KErr;

use std::mem::size_of;

#[test]
fn svec1() {
    let vec = SVec4::<i32>::new();
    let ai = vec.push(0).unwrap();
    let bi = vec.push(1).unwrap();
    eprintln!("{} {}",vec[ai],vec[bi]);
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
    assert_eq!(vec.to_string(), "SVec4[ 1, 2, 3 ]");

    assert_eq!(vec.pop(), 3);
    assert_eq!(vec.to_string(), "SVec4[ 1, 2 ]");
    vec.push(4).unwrap();
    assert_eq!(vec.to_string(), "SVec4[ 1, 2, 4 ]");

    vec[0] = 5;
    assert_eq!(vec.to_string(), "SVec4[ 5, 2, 4 ]");

    vec.insert(1,6);
    assert_eq!(vec.to_string(), "SVec4[ 5, 6, 2, 4 ]");

    vec.remove(1);
    assert_eq!(vec.to_string(), "SVec4[ 5, 2, 4 ]");

    let x = &vec[1];   // Get shared access.
    assert_eq!(x, &2);

    vec.push(7).unwrap(); // Append is still ok even with 'x' around.

    vec[2] = 8;           // Mutation is not allowed while 'x' exists.
    vec.remove(2);
    vec.pop();
    vec.pop();
    assert_eq!(vec.to_string(), "SVec4[ 5 ]");

    // assert_eq!(x, &2); // Uncomment to test borrow check.

    let mut sum = 0i32;
    let iter = vec.iter();
    vec.push(8).unwrap();  // Appends are always ok.  The new item doesn't get included in the iterator.
    //vec.pop(); // Mutation is not ok while iter is still being used.
    for x in iter { sum+=x; }
    assert_eq!(sum, 5);

    let mut sum = 0i32;
    let iter = (&vec).into_iter();
    vec.push(9).unwrap();
    //vec.pop(); // Mutation is not ok while iter is still being used.
    for x in iter { sum+=x; }
    assert_eq!(sum, 13);

    vec.push(10).unwrap();
    vec.pop(); // Mutation is ok again.

    let iter = vec.iter_mut();
    //vec.pop();  // Not allowed
    for x in iter { *x+=10; }
    assert_eq!(vec.to_string(), "SVec4[ 15, 18, 19 ]");

    let iter = (&mut vec).into_iter();
    //vec.pop();  // Not allowed
    for x in iter { *x+=10; }
    assert_eq!(vec.to_string(), "SVec4[ 25, 28, 29 ]");

    vec.push(11).unwrap();
    vec.pop(); // Mutation is ok again.
    assert_eq!(vec.to_string(), "SVec4[ 25, 28, 29 ]");
}

#[test]
fn as_str() {
    let vec = SVec16::<u8>::new();
    vec.push(b'H').unwrap();
    vec.push(b'e').unwrap();
    vec.push(b'l').unwrap();
    vec.push(b'l').unwrap();
    vec.push(b'o').unwrap();
    assert_eq!(vec.as_str().unwrap(), "Hello");

    let vec = SVec16::<i32>::new();
    vec.push(0).unwrap();
    // assert_eq!(vec.as_str().unwrap(), "...");  // 'as_str()' doesn't exist for non-u8 types.
}

