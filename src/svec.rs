use kerr::KErr;
use std::slice;
use std::ops::{Index, IndexMut};


// This is how you can manually convert a value to a type, and call a type-parameterized function:
// pub fn cap<T>(_:&T) -> usize where T:SVec { T::cap() }

pub trait SVec : Index<usize, Output=<Self as SVec>::Item> + IndexMut<usize> {  // Must use that crazy syntax to tell the compiler that the associated types are equal.
    type Item;

    // ---- Append-Only Interface ----
    // If you ONLY use this section, you can't have bugs.
    fn new() -> Self where Self:Sized;  // https://github.com/rust-lang/rfcs/blob/master/text/0546-Self-not-sized-by-default.md
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
    fn clear(&mut self);
    fn pop(&mut self) -> Self::Item;
    //fn set(&mut self, i:usize, t:Self::Item);
    fn insert(&mut self, i:usize, t:Self::Item);
    fn remove(&mut self, i:usize) -> Self::Item;
    fn reverse(&mut self);
    fn as_slice_mut(&mut self) -> &mut [Self::Item];
    //fn iter_mut(&mut self) -> impl Iterator<Item=&mut Self::Item>  // Currently not able to return 'impl Trait' from Trait methods.  https://github.com/rust-lang/rfcs/blob/master/text/1522-conservative-impl-trait.md#limitation-to-freeinherent-functions
    //fn iter_mut_dyn<'a>(&'a mut self) -> Box<dyn Iterator<Item=&mut Self::Item> + 'a>;
    fn iter_mut<'a>(&'a mut self) -> slice::IterMut<'a,Self::Item>;

    // Due to our stack allocation and aversion to copying of data, I can't do a standard implementation of IntoIterator because it takes ownership.
    // Here is a similar thing that takes '&mut' instead and returns owned objects.
    fn iter_owned<'a>(&'a mut self) -> IntoIter<'a,Self::Item> where Self:Sized {
        IntoIter::new(self)
    }

    // ---- Debugging ----
    //fn dataptr(&self) -> *const Self::Item;
}




macro_rules! def_stackvec {
    ( $size:expr, $svec:ident, $sstring:ident ) => {
        pub struct $svec<T> {
            //data: UnsafeCell<[Option<T>; $size]>,  // I'm using Option mainly for efficient drops.  Also enables me to hardcode the initial values.  NEVERMIND, if i do this, i can't slice efficiently.
            data: ManuallyDrop<UnsafeCell<[T; $size]>>,
            length: Cell<usize>,
        }
        impl<T> SVec for $svec<T> {
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

            fn clear(&mut self) {
                let mut length = self.length.get();
                while length>0 {
                    unsafe { ptr::drop_in_place(&mut (*self.data.get())[length-1]); }
                    length-=1; self.length.set(length);
                }
            }

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

            fn reverse(&mut self) {
                let mut i=0; let mut j=self.length.get()-1;
                let aptr = self.data.get();
                while i<j {
                    unsafe {
                        // Cannot deref aptr outside the loop because of borrow checker.
                        let i_p = &mut (*aptr)[i];
                        let j_p = &mut (*aptr)[j];
                        mem::swap(i_p, j_p);
                    }

                    i+=1; j-=1;
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

            //fn dataptr(&self) -> *const T { self.data.get() as *const T }  // For Debugging
        }
        impl<T> $svec<T> {
            // I can't put this in the trait interface because I don't have a way of specifying $svec.
            // I can refactor when we have const_generics.
            pub fn new_of<U>(&self) -> $svec<U> { $svec::<U>::new() }

            pub fn from_iter_err<I>(iter:I) -> Result<Self,KErr> where I:IntoIterator<Item=T> {
                let out = Self::new();
                for t in iter { out.push(t)?; }
                Ok(out)
            }
        }
        // Maybe place this into the above impl when Type Equality Bounding is implemented):
        // https://github.com/rust-lang/rust/issues/20041
        impl $svec<u8> {
            #[inline]
            pub fn as_str(&self) -> Result<&str, KErr> {
                std::str::from_utf8(self.as_slice()).map_err(|_| KErr::new("Utf8Error"))
            }
        }

        impl<T> Drop for $svec<T> {
            fn drop(&mut self) {
                //eprintln!("svec drop start");
                self.clear();
                //eprintln!("svec drop end");
            }
        }

        impl<T> Index<usize> for $svec<T> {
            type Output = T;
            fn index(&self, index:usize) -> &Self::Output {
                if index>=self.length.get() { panic!("out-of-bounds"); }
                unsafe { &(*self.data.get())[index] }
            }
        }
        impl<T> IndexMut<usize> for $svec<T> {
            fn index_mut(&mut self, index:usize) -> &mut Self::Output {
                if index>=self.length.get() { panic!("out-of-bounds"); }
                unsafe { &mut (*self.data.get())[index] }
            }
        }

        //////// We can't do this because into_iter() wants to take ownership of 'self', and that doesn't play nicely with stack allocations.
        // impl<T> IntoIterator for $svec<T> {
        //     type Item = T;
        //     type IntoIter = $intoiter<T>;
        //     fn into_iter(self) -> Self::IntoIter {
        //         // mem::transmute::<Self, Self::IntoIter>(self)  // Can't transmute generics cuz compiler doesn't know their sizes (even though it totally could).
        // 
        //         // Here's my own transmute implementation:
        //         assert_eq!(mem::size_of::<$svec<T>>(), mem::size_of::<$intoiter<T>>());
        //         unsafe {
        //             let iter = ptr::read(&self as *const $svec<T> as *const $intoiter<T>);
        //             mem::forget(self);
        //             iter
        //         }
        //     }
        // }
        impl<'a,T> IntoIterator for &'a $svec<T> {
            type Item = &'a T;
            type IntoIter = slice::Iter<'a,T>;
            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }
        impl<'a,T> IntoIterator for &'a mut $svec<T> {
            type Item = &'a mut T;
            type IntoIter = slice::IterMut<'a,T>;
            fn into_iter(self) -> Self::IntoIter {
                self.iter_mut()
            }
        }

        impl<T> FromIterator<T> for $svec<T> {
            fn from_iter<I>(iter:I) -> Self where I:IntoIterator<Item=T> {
                Self::from_iter_err(iter).unwrap()
            }
        }

        impl<T,U> PartialEq<U> for $svec<T> where T:PartialEq, U:SVec<Item=T, Output=T> {
            fn eq(&self, other:&U) -> bool {
                if self.len()!=other.len() { return false }
                for i in 0..self.len() {
                    if self[i]!=other[i] { return false }
                }
                true
            }
        }

        // I can't figure out how to turn this into a blanket implementation...
        // The difficulty is maybe because the Trait and the Type are both parameterized,
        // and so i can't figure out how to specify those parameters along with the extra
        // Display constraint...  I'm sure it's easy, but I can't figure it out.
        impl<T> fmt::Display for $svec<T> where T:fmt::Display {
            fn fmt(&self, f:&mut fmt::Formatter) -> Result<(), fmt::Error> {
                let mut nonempty = false;
                write!(f, "[")?;
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
        impl<T> fmt::Debug for $svec<T> where T:fmt::Debug {
            fn fmt(&self, f:&mut fmt::Formatter) -> Result<(), fmt::Error> {
                let mut nonempty = false;
                write!(f, "{}[", stringify!($svec))?;
                for t in self.as_slice().iter() {
                    if nonempty { write!(f, ",")?; }
                    nonempty = true;
                    write!(f, " {:?}", t)?;
                }
                if nonempty { write!(f, " ")?; }
                write!(f, "]")?;
                Ok(())
            }
        }


        pub type $sstring = $svec<u8>;
        // ---- I wanted to use this newtype, but it runs 50x slower than the above alias!!!
        //      The compiler does a *horrible* job of handling this simple case...
        //      I think it's because I was hitting a very perfect scenario where the compiler
        //      produced code with almost no jumps and very localized code and data, which
        //      resulted in some extremely fast benches.  Combined with the fact that the
        //      compiler doesn't try to inline functions across crates, and the extra fn call
        //      was a killer for the tight-loop.
        // pub struct $sstring(pub $svec<u8>)
        // impl $sstring {
        //     pub fn new() -> Self { Self($svec::new()) }
        // }
        // impl SString for $sstring {
        //     #[inline]
        //     fn cap() -> usize { $svec::<u8>::cap() }
        //     #[inline]
        //     fn len(&self) -> usize { self.0.len() }
        //     #[inline]
        //     fn push(&self, b:u8) -> Result<usize,KErr> { self.0.push(b) }
        // }
    }
}

pub struct IntoIter<'a,T>(&'a mut dyn SVec<Item=T, Output=T>);
impl<'a,T> Iterator for IntoIter<'a,T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.len()==0 { return None }
        Some(self.0.pop())
    }
}
impl<T> IntoIter<'_,T> {
    pub fn new<'a>(svec:&'a mut dyn SVec<Item=T, Output=T>) -> IntoIter<'a,T> {
        IntoIter(svec)
    }

    //pub fn dataptr(&self) -> *const T { self.0.dataptr() }
}

