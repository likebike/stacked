use kerr::KErr;
use std::slice;
use std::ops::{Index, IndexMut};


// This is how you can manually convert a value to a type, and call a type-parameterized function:
// pub fn cap<T>(_:&T) -> usize where T:SVec { T::cap() }

pub trait SVec : Sized + Drop + Index<usize> + IndexMut<usize> {  // https://github.com/rust-lang/rfcs/blob/master/text/0546-Self-not-sized-by-default.md
    type Item;

    // ---- Append-Only Interface ----
    // If you ONLY use this section, you can't have bugs.
    fn new() -> Self;
    fn cap(&self) -> usize;
    fn len(&self) -> usize;
    fn push(&self, t:Self::Item) -> Result<usize,KErr>;
    //fn get(&self, i:usize) -> &Self::Item;
    //fn get_copy(&self, i:usize) -> Self::Item where Self::Item:Copy;
    //fn iter(&self) -> impl Iterator<Item=&Self::Item>;  // See comment on iter_mut() .
    //fn iter_dyn<'a>(&'a self) -> Box<dyn Iterator<Item=&Self::Item> + 'a>;
    fn iter<'a>(&'a self) -> slice::Iter<'a,Self::Item>;
    fn as_slice(&self) -> &[Self::Item];
    //fn as_str(&self) -> Result<&str, KErr> where Self::Item=u8;

    // ---- Mutation Interface ----
    // If you use any of this section AT ALL, it is up to you to keep the bugs out.
    fn pop(&mut self) -> Self::Item;
    //fn set(&mut self, i:usize, t:Self::Item);
    fn insert(&mut self, i:usize, t:Self::Item);
    fn remove(&mut self, i:usize) -> Self::Item;
    fn as_slice_mut(&mut self) -> &mut [Self::Item];
    //fn iter_mut(&mut self) -> impl Iterator<Item=&mut Self::Item>  // Currently not able to return 'impl Trait' from Trait methods.  https://github.com/rust-lang/rfcs/blob/master/text/1522-conservative-impl-trait.md#limitation-to-freeinherent-functions
    //fn iter_mut_dyn<'a>(&'a mut self) -> Box<dyn Iterator<Item=&mut Self::Item> + 'a>;
    fn iter_mut<'a>(&'a mut self) -> slice::IterMut<'a,Self::Item>;
}




macro_rules! def_stackvec {
    ( $name:ident, $strname:ident, $size:expr ) => {
        pub struct $name<T> {
            //data: UnsafeCell<[Option<T>; $size]>,  // I'm using Option mainly for efficient drops.  Also enables me to hardcode the initial values.  NEVERMIND, if i do this, i can't slice efficiently.
            data: ManuallyDrop<UnsafeCell<[T; $size]>>,
            length: Cell<usize>,
        }
        impl<T> SVec for $name<T> {
            type Item = T;

            #[inline]
            fn new() -> Self {
                Self{ data: ManuallyDrop::new(UnsafeCell::new(unsafe { mem::zeroed() })),
                      length: Cell::new(0) }
            }

            #[inline]
            fn cap(&self) -> usize { $size }
            #[inline]
            fn len(&self) -> usize { self.length.get() }

            fn push(&self, t:T) -> Result<usize,KErr> {
                let i = self.length.get();
                if i>=self.cap() { return Err(KErr::new("overflow")); }
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

            // #[inline]
            // fn get(&self, i:usize) -> &T {
            //     if i>=self.length.get() { panic!("out-of-bounds"); }
            //     unsafe { &(*self.data.get())[i] }
            // }
            // #[inline]
            // fn get_copy(&self, i:usize) -> T where T:Copy {
            //     if i>=self.length.get() { panic!("out-of-bounds"); }
            //     unsafe { (*self.data.get())[i] }
            // }
            // fn set(&mut self, i:usize, t:T) {
            //     let len = self.length.get();
            //     if i>len { panic!("out-of-bounds") }
            //     if i==len {
            //         self.push(t).unwrap();
            //         return;
            //     }
            //     unsafe { (*self.data.get())[i] = t; }
            // }

            fn insert(&mut self, i:usize, t:T) {
                let len = self.length.get();
                if i>len { panic!("out-of-bounds"); }
                if i>=self.cap() { panic!("overflow"); }

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
            #[inline]
            fn as_slice_mut(&mut self) -> &mut [T] {
                unsafe { &mut (*self.data.get())[..self.len()] }
            }


            //pub fn iter(&self) -> impl Iterator<Item=&T> {  // Currently not able to return 'impl Trait' from trait methods.  Maybe someday...
            #[inline]
            fn iter<'a>(&'a self) -> slice::Iter<'a,T> {
                self.as_slice().iter()
            }
            //pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut T> {  // Currently not able to return 'impl Trait' from trait methods.
            #[inline]
            fn iter_mut<'a>(&'a mut self) -> slice::IterMut<'a,T> {
                self.as_slice_mut().iter_mut()
            }
            // #[inline]
            // fn iter_dyn<'a>(&'a self) -> Box<dyn Iterator<Item=&T> + 'a> {
            //     box self.iter()
            // }
            // #[inline]
            // fn iter_mut_dyn<'a>(&'a mut self) -> Box<dyn Iterator<Item=&mut T> + 'a> {
            //     box self.iter_mut()
            // }
        }
        // Maybe place this into the above impl when Type Equality Bounding is implemented):
        // https://github.com/rust-lang/rust/issues/20041
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

        impl<T> Index<usize> for $name<T> {
            type Output = T;
            fn index(&self, index:usize) -> &Self::Output {
                if index>=self.length.get() { panic!("out-of-bounds"); }
                unsafe { &(*self.data.get())[index] }
            }
        }
        impl<T> IndexMut<usize> for $name<T> {
            fn index_mut(&mut self, index:usize) -> &mut Self::Output {
                if index>=self.length.get() { panic!("out-of-bounds"); }
                unsafe { &mut (*self.data.get())[index] }
            }
        }

        impl<T> IntoIterator for $name<T> {
            type Item = T;
            type IntoIter = std::option::IntoIter<T>;
            fn into_iter(self) -> Self::IntoIter {
                panic!("Do not convert an 'SVec' object to an Iterator directly -- use '&SVec' or '&mut SVec' or .iter() or .iter_mut() instead.");
            }
        }
        impl<'a,T> IntoIterator for &'a $name<T> {
            type Item = &'a T;
            type IntoIter = slice::Iter<'a,T>;
            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }
        impl<'a,T> IntoIterator for &'a mut $name<T> {
            type Item = &'a mut T;
            type IntoIter = slice::IterMut<'a,T>;
            fn into_iter(self) -> Self::IntoIter {
                self.iter_mut()
            }
        }

        //impl<T> PartialEq for $name<T> where T:PartialEq {
        //    fn eq(&self, other:&Self) -> bool {
        //        if self.len()!=other.len() { return false }
        //        unimplemented!();
        //    }
        //}

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

