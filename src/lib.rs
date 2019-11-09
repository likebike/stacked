#![feature(test, box_syntax)]

extern crate test;

#[macro_use]
mod svec;
// mod sstring;  // Newtypes are way too expensive!  Just alias to SVec instead.
// mod sref;     // I'll probably do this later.  Right now, it's just simpler-to-understand and more efficient to just use raw indexes.

pub use self::svec::{SVec, IntoIter};

use kerr::KErr;

use std::fmt;
use std::mem::{self, ManuallyDrop};
use std::ptr;
use std::cell::{UnsafeCell, Cell};
use std::slice;
use std::ops::{Index, IndexMut};
use std::iter::FromIterator;


// I am looking forward to const_generics : https://github.com/rust-lang/rust/issues/44580

def_stackvec!(   1,    SVec1,    SString1);
def_stackvec!(   2,    SVec2,    SString2);
def_stackvec!(   4,    SVec4,    SString4);
def_stackvec!(   8,    SVec8,    SString8);
def_stackvec!(  16,   SVec16,   SString16);
def_stackvec!(  32,   SVec32,   SString32);
def_stackvec!(  64,   SVec64,   SString64);
def_stackvec!( 128,  SVec128,  SString128);
def_stackvec!( 256,  SVec256,  SString256);
def_stackvec!( 512,  SVec512,  SString512);
def_stackvec!(1024, SVec1024, SString1024);
def_stackvec!(2048, SVec2048, SString2048);
def_stackvec!(4096, SVec4096, SString4096);
def_stackvec!(8192, SVec8192, SString8192);


#[cfg(test)]
mod internal_tests {
    use super::*;

    // An experiment, to see how 'drop' works when overwriting values,
    // and also to verify that we really are mutating the memory we expect:

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

    impl<T> SVec4<T> where T:PartialEq {
        fn zet(&self, i:usize, t:T) {  // Intentionally unsafe design -- I'm mutating via a shared reference so I can verify that I'm really modifying the memory i expect.
            unsafe { (*self.data.get())[i] = t; }
        }
    }
    //impl<T> Drop for SVec4<T> {
    //    fn drop(&mut self) {
    //        eprintln!("in stackvec drop");
    //    }
    //}

    #[test]
    fn svec3() {
        eprintln!("I expect to see: 1 -1 0 START 4 3 2 -11 END");
        let vec = SVec4::<Dropper>::new();
        assert_eq!(vec.len(),0);

        let i0 = vec.push(Dropper(1)).unwrap();
        assert_eq!(i0,0);
        let ref0 = &vec[i0];
        assert_eq!(ref0.0,1);

        vec.push(Dropper(2)).unwrap();

        vec.zet(0, Dropper(-1));
        assert_eq!(ref0.0,-1);

        vec.zet(0, Dropper(-11));
        assert_eq!(ref0.0,-11);

        vec.zet(3, Dropper(-3));  // Treats existing zero-bytes as a Dropper and drops it.
                                  // We're lucky zeroed memory happens to be a valid i32, otherwise BAD THINGS could happen!
                                  // This item's drop() won't be called because SVec assumes it has not been initialized!

        vec.push(Dropper(3)).unwrap();
        vec.push(Dropper(4)).unwrap();
    }



    use test::{Bencher, black_box};

    #[bench]
    fn svec1(b:&mut Bencher) {
        b.iter(|| {
            let a = 333; black_box(a);
            for _ in 1..100 {
                let v = SVec32::<u8>::new();
                let cap = v.cap();  //SVec32::<u8>::cap();
                while v.len()<cap { v.push(b'1').unwrap(); }

                black_box(v);
            }
            let z = 444; black_box(z);
        });
    }

}

