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

