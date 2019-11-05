// So I can use benchmarks:
#![feature(test)]
extern crate test;

mod svec;
// mod sstring;  // Newtypes are way too expensive!  Just alias to SVec instead.
// mod sref;     // I'll probably do this later.  Right now, it's just simpler-to-understand and more efficient to just use raw indexes.

pub use self::svec::SVec;

use kerr::KErr;

use std::fmt;
use std::mem::{self, ManuallyDrop};
use std::ptr;
use std::cell::{UnsafeCell, Cell};

macro_rules! def_stackvec {
    ( $name:ident, $strname:ident, $size:expr ) => {
        pub struct $name<T> {
            //data: UnsafeCell<[Option<T>; $size]>,  // I'm using Option mainly for efficient drops.  Also enables me to hardcode the initial values.  NEVERMIND, if i do this, i can't slice efficiently.
            data: ManuallyDrop<UnsafeCell<[T; $size]>>,
            length: Cell<usize>,
        }
        impl<T> SVec<T> for $name<T> {
            #[inline]
            fn new() -> Self {
                Self{ data: ManuallyDrop::new(UnsafeCell::new(unsafe { mem::zeroed() })),
                      length: Cell::new(0) }
            }

            #[inline]
            fn cap() -> usize { $size }
            #[inline]
            fn len(&self) -> usize { self.length.get() }

            fn push(&self, t:T) -> Result<usize,KErr> {
                let i = self.length.get();
                if i>=Self::cap() { return Err(KErr::new("overflow")); }
                unsafe { ptr::write(&mut (*self.data.get())[i], t); }
                self.length.set(i+1);
                Ok(i)
            }
            fn pop(&mut self) -> T {
                let len = self.length.get();
                if len==0 { panic!("underflow"); }
                let t = unsafe { ptr::read( &(*self.data.get())[len-1] ) };
                self.length.set(len-1);
                t
            }

            #[inline]
            fn get(&self, i:usize) -> &T {
                if i>=self.length.get() { panic!("out-of-bounds"); }
                unsafe { &(*self.data.get())[i] }
            }
            #[inline]
            fn get_copy(&self, i:usize) -> T where T:Copy {
                if i>=self.length.get() { panic!("out-of-bounds"); }
                unsafe { (*self.data.get())[i] }
            }
            fn set(&mut self, i:usize, t:T) {
                let len = self.length.get();
                if i>len { panic!("out-of-bounds") }
                if i==len {
                    self.push(t).unwrap();
                    return;
                }
                unsafe { (*self.data.get())[i] = t; }
            }

            fn insert(&mut self, i:usize, t:T) {
                let len = self.length.get();
                if i>len { panic!("out-of-bounds"); }
                if i>=Self::cap() { panic!("overflow"); }

                unsafe {
                    let p = &mut (*self.data.get())[i] as *mut T;
                    ptr::copy(p, p.offset(1), len-i);
                    ptr::write(p, t);
                    self.length.set(len+1);
                }
            }
            fn remove(&mut self, i:usize) -> T {
                let len = self.length.get();
                if i>=len { panic!("out-of-bounds"); }

                unsafe {
                    let p = &mut (*self.data.get())[i] as *mut T;
                    let t = ptr::read(p);
                    self.length.set(len-1);
                    ptr::copy(p.offset(1), p, len-i-1);
                    t
                }
            }

            #[inline]
            fn as_slice(&self) -> &[T] {
                unsafe { &(*self.data.get())[..self.len()] }
            }
        }
        impl $name<u8> {
            #[inline]
            pub fn as_str(&self) -> Result<&str, KErr> {
                std::str::from_utf8(self.as_slice()).map_err(|_| KErr::new("Utf8Error"))
            }
        }
        impl<T> Drop for $name<T> {
            fn drop(&mut self) {
                //eprintln!("svec drop start");

                let mut length = self.length.get();
                while length>0 {
                    unsafe { ptr::drop_in_place(&mut (*self.data.get())[length-1]); }
                    length-=1; self.length.set(length);
                }

                //eprintln!("svec drop end");
            }
        }
        impl<T> PartialEq for $name<T> where T:PartialEq {
            fn eq(&self, other:&Self) -> bool {
                if self.len()!=other.len() { return false }
                unimplemented!();
            }
        }
        // I can't figure out how to turn this into a blanket implementation...
        // The difficulty is maybe because the Trait and the Type are both parameterized,
        // and so i can't figure out how to specify those parameters along with the extra
        // Display constraint...  I'm sure it's easy, but I can't figure it out.
        impl<T> fmt::Display for $name<T> where T:fmt::Display {
            fn fmt(&self, f:&mut fmt::Formatter) -> Result<(), fmt::Error> {
                let mut nonempty = false;
                write!(f, "{}[", stringify!($name))?;
                for t in self.as_slice().iter() {
                    if nonempty { write!(f, ",")?; }
                    nonempty = true;
                    write!(f, " {}", t)?;
                }
                if nonempty { write!(f, " ")?; }
                write!(f, "]")?;
                Ok(())
            }
        }

        pub type $strname = $name<u8>;
        // ---- I wanted to use this newtype, but it runs 50x slower than the above alias!!!
        //      The compiler does a *horrible* job of handling this simple case...
        //      I think it's because I was hitting a very perfect scenario where the compiler
        //      produced code with almost no jumps and very localized code and data, which
        //      resulted in some extremely fast benches.  Combined with the fact that the
        //      compiler doesn't try to inline functions across crates, and the extra fn call
        //      was a killer for the tight-loop.
        // pub struct $strname(pub $name<u8>)
        // impl $strname {
        //     pub fn new() -> Self { Self($name::new()) }
        // }
        // impl SString for $strname {
        //     #[inline]
        //     fn cap() -> usize { $name::<u8>::cap() }
        //     #[inline]
        //     fn len(&self) -> usize { self.0.len() }
        //     #[inline]
        //     fn push(&self, b:u8) -> Result<usize,KErr> { self.0.push(b) }
        // }
    }
}

def_stackvec!(   SVec2,    SString2,    2);
def_stackvec!(   SVec4,    SString4,    4);
def_stackvec!(   SVec8,    SString8,    8);
def_stackvec!(  SVec16,   SString16,   16);
def_stackvec!(  SVec32,   SString32,   32);
def_stackvec!(  SVec64,   SString64,   64);
def_stackvec!( SVec128,  SString128,  128);
def_stackvec!( SVec256,  SString256,  256);
def_stackvec!( SVec512,  SString512,  512);
def_stackvec!(SVec1024, SString1024, 1024);


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
        fn zet(&self, i:usize, t:T) {
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
        let ref0 = vec.get(i0);
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
                let cap = SVec32::<u8>::cap();
                while v.len()<cap { v.push(b'1').unwrap(); }

                black_box(v);
            }
            let z = 444; black_box(z);
        });
    }

}

