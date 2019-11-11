#![feature(backtrace)]

use stacked::{SVec, SVec4, SVec16};

use kerr::KErr;

use std::mem::size_of;
use std::backtrace::Backtrace;

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



#[derive(PartialEq, Debug)]
struct Dropper(i32);
impl Drop for Dropper {
    fn drop(&mut self) {
        eprintln!("in Dropper.drop: {}", self.0);
        if self.0%10==0 { eprintln!("{}",Backtrace::capture()); }
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
    assert_eq!(vec.to_string(), "[]");
    vec.push(1).unwrap();
    vec.push(2).unwrap();
    vec.push(3).unwrap();
    assert_eq!(vec.to_string(), "[ 1, 2, 3 ]");

    assert_eq!(vec.pop(), 3);
    assert_eq!(vec.to_string(), "[ 1, 2 ]");
    vec.push(4).unwrap();
    assert_eq!(vec.to_string(), "[ 1, 2, 4 ]");

    vec[0] = 5;
    assert_eq!(vec.to_string(), "[ 5, 2, 4 ]");

    vec.insert(1,6);
    assert_eq!(vec.to_string(), "[ 5, 6, 2, 4 ]");

    vec.remove(1);
    assert_eq!(vec.to_string(), "[ 5, 2, 4 ]");

    let x = &vec[1];   // Get shared access.
    assert_eq!(x, &2);

    vec.push(7).unwrap(); // Append is still ok even with 'x' around.

    vec[2] = 8;           // Mutation is not allowed while 'x' exists.
    vec.remove(2);
    vec.pop();
    vec.pop();
    assert_eq!(vec.to_string(), "[ 5 ]");

    // assert_eq!(x, &2); // Uncomment to test borrow check.

    let mut sum = 0i32;
    let iter = vec.iter();
    vec.push(8).unwrap();  // Appends are always allowed.  The new item doesn't get included in the iterator.
    //vec.pop(); // Mutation is not allowed while iter is still being used.
    for x in iter { sum+=x; }
    assert_eq!(sum, 5);

    let mut sum = 0i32;
    let iter = (&vec).into_iter();
    vec.push(9).unwrap();
    //vec.pop(); // Mutation is not allowed while iter is still being used.
    for x in iter { sum+=x; }
    assert_eq!(sum, 13);

    let mut sum = 0i32;
    for x in &vec { sum+=x; }
    assert_eq!(sum, 22);

    vec.push(10).unwrap();
    vec.pop(); // Mutation is allowed again.

    let iter = vec.iter_mut();
    //vec.pop();  // Not allowed
    for x in iter { *x+=10; }
    assert_eq!(vec.to_string(), "[ 15, 18, 19 ]");

    let iter = (&mut vec).into_iter();
    //vec.pop();  // Not allowed
    for x in iter { *x+=10; }
    assert_eq!(vec.to_string(), "[ 25, 28, 29 ]");

    for x in &mut vec { *x+=10; }
    assert_eq!(vec.to_string(), "[ 35, 38, 39 ]");

    vec.push(11).unwrap();
    vec.pop(); // Mutation is allowed again.
    assert_eq!(vec.to_string(), "[ 35, 38, 39 ]");
    assert_eq!(format!("{:?}",vec), "SVec4[ 35, 38, 39 ]");

    //for x in vec {}  // Nice error message.
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

#[test]
fn partialeq() {
    let a = SVec4::<u8>::new();
    let b = SVec16::<u8>::new();

    assert_eq!(a.eq(&b), true);
    assert_eq!(b.eq(&a), true);

    a.push(0).unwrap();

    assert_eq!(a.eq(&b), false);
    assert_eq!(b.eq(&a), false);

    b.push(0).unwrap();

    assert_eq!(a.eq(&b), true);
    assert_eq!(b.eq(&a), true);
}

#[test]
fn fromiter() {
    // Vec works as expected:
    {
        let v = vec![Dropper(10), Dropper(11)];
        for x in v {
            println!("in v loop: {}",x.0);
        }
        // println!("{}",v.len());  // 'v' already moved by 'for'
        println!("done with v loop");
    }

    // Array does NOT iterate owned values:
    {
        let a = [Dropper(20), Dropper(21)];
        for x in a.into_iter() {
            println!("in a loop: {}",x.0);
        }
        println!("{}",a.len());  // 'a' NOT moved by 'for'
        println!("done with a loop");
    }

    {
        let mut s = SVec4::<Dropper>::new();
        s.push(Dropper(30)).unwrap();
        s.push(Dropper(31)).unwrap();
        for x in s.iter_owned() {
            println!("in s loop: {}",x.0);
        }
        // println!("{}",s.len());  // 's' already moved by 'for'
        println!("done with s loop");
    }

    {
        let mut s = SVec4::<Dropper>::new();
        println!("addr of s: {:?}  dataptr: {:?}", &s as *const _, "s.dataptr()");
        s.push(Dropper(40)).unwrap();
        s.push(Dropper(41)).unwrap();
        for x in s.iter_owned() {
            println!("in s2 loop: {}",x.0);
            break;
        }
        // println!("{}",s.len());  // 's' already moved by 'for'
        println!("done with s2 loop");
    }

    {
        println!("Testing unsound intoiter lifetime:");
        #[inline(never)]
        fn output_an_intoiter() /*-> IntoIter<'static,Dropper>*/ {
            let s = SVec4::<Dropper>::new();
            println!("addr of s: {:?}  dataptr: {:?}", &s as *const _, "s.dataptr()");
            s.push(Dropper(50)).unwrap();
            s.push(Dropper(51)).unwrap();

            let mut t = s;
            println!("addr of t: {:?}  dataptr: {:?}", &t as *const _, "t.dataptr()");

            t = SVec4::<Dropper>::new();
            println!("addr of t: {:?}  dataptr: {:?}", &t as *const _, "t.dataptr()");
            t.push(Dropper(60)).unwrap();
            t.push(Dropper(61)).unwrap();
            println!("addr of t: {:?}  dataptr: {:?}", &t as *const _, "t.dataptr()");

            let intoiter1 = t.iter_owned();
            println!("addr of intoiter1: {:?}  dataptr: {:?}", &intoiter1 as *const _, "intoiter1.dataptr()");
        //    intoiter1    // Yay, the compiler prevents us from returning this iterator out of the function.
        }
        let _intoiter2 = output_an_intoiter();
        println!("after drop of svec -- only intoiter remains...");
        //println!("addr of intoiter2: {:?}  dataptr: {:?}", &_intoiter2 as *const _, "_intoiter2.dataptr()");
        //for x in _intoiter2 {
        //    println!("in unsound intoiter loop: {}",x.0);
        //}
    }

    //let b : SVec4<Dropper> = [Dropper(1), Dropper(2)].into_iter().collect();
}

#[test]
fn reverse() {
    let mut s = SVec4::<Dropper>::new();
    s.push(Dropper(1)).unwrap();
    s.push(Dropper(2)).unwrap();
    println!("before: {:?}",s);
    s.reverse();
    println!("reversed: {:?}",s);
    s.push(Dropper(3)).unwrap();
    println!("before 2: {:?}",s);
    s.reverse();
    println!("reversed 2: {:?}",s);
}

#[test]
fn literal() {
    let svec : SVec16<_> = vec![1,2,3].into();
    assert_eq!(svec.len(), 3);
    assert_eq!(svec.to_string(), "[ 1, 2, 3 ]");
}

#[test]
fn collect() {
    let svec : SVec16<_> = vec![1,2,3].into_iter().collect();
    assert_eq!(svec.len(), 3);
    assert_eq!(svec.to_string(), "[ 1, 2, 3 ]");
}

