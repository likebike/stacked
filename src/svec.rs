use kerr::KErr;
use std::ops::{Index, IndexMut};


// For future reference: This is how you can manually convert a value to a type, and call a type-parameterized function:
// pub fn cap<T>(_:&T) -> usize where T:SVec { T::cap() }

pub trait SVec : Index<usize, Output=<Self as SVec>::Item> + IndexMut<usize> {  // Must use that crazy syntax to tell the compiler that the associated types are equal.
    type Item;

    // ---- Append-Only Interface ----
    // If you ONLY use this section, you can't have bugs.
    fn new() -> Self where Self:Sized;  // https://github.com/rust-lang/rfcs/blob/master/text/0546-Self-not-sized-by-default.md
    fn cap(&self) -> usize;
    fn len(&self) -> usize;
    fn push(&mut self, t:Self::Item) -> Result<usize,KErr>;
    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item=&'a Self::Item> + 'a>;
    fn iter_owned<'a>(&'a mut self) -> Box<dyn Iterator<Item=Self::Item> + 'a>;
    

    // ---- Mutation Interface ----
    // If you use any of this section AT ALL, it is up to you to keep the bugs out.
    fn clear(&mut self);
    fn pop(&mut self) -> Self::Item;
    fn insert(&mut self, i:usize, t:Self::Item);
    fn remove(&mut self, i:usize) -> Self::Item;
    fn reverse(&mut self);
    fn iter_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item=&'a mut Self::Item> + 'a>;

    // ---- Internals ----
    fn as_opt_slice(&self) -> &[Option<Self::Item>];
    fn as_opt_slice_mut(&mut self) -> &mut[Option<Self::Item>];
}



macro_rules! def_stackvec {
    ( $size:expr, $svec:ident, $sstring:ident ) => {
        pub struct $svec<T> {
            data: [Option<T>; $size],
            length: usize,
        }
        impl<T> SVec for $svec<T> {
            type Item = T;

            #[inline]
            fn new() -> Self {
                Self{ data:/*unsafe { mem::zeroed() },*/ [None; $size],  // const_in_array_repeat_expression
                      length:0 }
            }

            #[inline]
            fn cap(&self) -> usize { $size }
            #[inline]
            fn len(&self) -> usize { self.length }

            fn clear(&mut self) {
                while self.length>0 {
                    self.data[self.length-1] = None;
                    self.length-=1;
                }
            }

            fn push(&mut self, t:T) -> Result<usize,KErr> {
                let i = self.length;
                if i>=Self::cap_of_type() { return Err(KErr::new("overflow")); }
                self.data[i] = Some(t);
                self.length+=1;
                Ok(i)
            }
            fn pop(&mut self) -> T {
                if self.length==0 { panic!("underflow"); }
                let t = self.data[self.length-1].take().unwrap();
                self.length-=1;
                t
            }

            fn insert(&mut self, i:usize, t:T) {
                if i>self.length { panic!("out-of-bounds"); }
                if i>=Self::cap_of_type() { panic!("overflow"); }

                unsafe {
                    let p = &mut self.data[i] as *mut Option<T>;
                    ptr::copy(p, p.offset(1), self.length-i);
                    ptr::write(p, Some(t));
                }
                self.length+=1;
            }
            fn remove(&mut self, i:usize) -> T {
                if i>=self.length { panic!("out-of-bounds"); }

                let t = self.data[i].take().unwrap();
                unsafe {
                    let p = &mut self.data[i] as *mut Option<T>;
                    self.length-=1;
                    ptr::copy(p.offset(1), p, self.length-i);       // Already subtracted 1 from length.
                    ptr::write(&mut self.data[self.length], None);  // Prevent double-drop.
                }
                t
            }

            fn reverse(&mut self) {
                let mut i=0; let mut j=self.length-1;
                let aptr = &mut self.data as *mut [Option<T>; $size];
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
            fn as_opt_slice(&self) -> &[Option<T>] {
                &self.data[..self.length]
            }
            #[inline]
            fn as_opt_slice_mut(&mut self) -> &mut [Option<T>] {
                &mut self.data[..self.length]
            }

            fn iter<'a>(&'a self) -> Box<dyn Iterator<Item=&'a T> + 'a> {
                Box::new(self.as_opt_slice().iter().map(|optref| optref.as_ref().unwrap()))
            }
            fn iter_mut<'a>(&'a mut self) -> Box<dyn Iterator<Item=&'a mut T> + 'a> {
                Box::new(self.as_opt_slice_mut().iter_mut().map(|optref| optref.as_mut().unwrap()))
            }
            // Due to our stack allocation and aversion to copying of data, I can't do a standard implementation of IntoIterator because it takes ownership.
            // Here is a similar thing that takes '&mut' instead and returns owned objects.
            fn iter_owned<'a>(&'a mut self) -> Box<dyn Iterator<Item=T> + 'a> {
                Box::new(IntoIter::new(self))
            }

        }
        impl<T> $svec<T> {
            #[inline]
            pub fn cap_of_type() -> usize { $size }

            // I can't put this in the trait interface because I don't have a way of specifying $svec.
            // I can refactor when we have const_generics.
            #[inline]
            pub fn new_of<U>(&self) -> $svec<U> { $svec::<U>::new() }

            // I'm not able to implement the TryFrom trait because of a conflict with a blanket impl.
            //     impl<T,I> TryFrom<I> for $svec<T> where I:IntoIterator<Item=T>
            // So that's why I'm putting this here:
            pub fn try_from_iter<I>(iter:I) -> Result<Self,KErr> where I:IntoIterator<Item=T> {
                let mut out = Self::new();
                for t in iter { out.push(t)?; }
                Ok(out)
                
            }
        }
        // Maybe place this into the above impl when Type Equality Bounding is implemented):
        // https://github.com/rust-lang/rust/issues/20041
        impl $svec<u8> {
            #[inline]
            pub fn as_string(&self) -> Result<String, KErr> {
                String::from_utf8(  self.iter().cloned().collect::<Vec<u8>>()  ).map_err(|_| KErr::new("Utf8Error"))
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
                if index>=self.length { panic!("out-of-bounds"); }
                self.data[index].as_ref().unwrap()
            }
        }
        impl<T> IndexMut<usize> for $svec<T> {
            fn index_mut(&mut self, index:usize) -> &mut Self::Output {
                if index>=self.length { panic!("out-of-bounds"); }
                self.data[index].as_mut().unwrap()
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
            type IntoIter = Box<dyn Iterator<Item=Self::Item> + 'a>;
            fn into_iter(self) -> Self::IntoIter {
                self.iter()
            }
        }
        impl<'a,T> IntoIterator for &'a mut $svec<T> {
            type Item = &'a mut T;
            type IntoIter = Box<dyn Iterator<Item=Self::Item> + 'a>;
            fn into_iter(self) -> Self::IntoIter {
                self.iter_mut()
            }
        }

        impl<T> iter::FromIterator<T> for $svec<T> {
            fn from_iter<I>(iter:I) -> Self where I:IntoIterator<Item=T> {
                Self::try_from_iter(iter).unwrap()
            }
        }
        // I'm surprised this isn't already handled by a blanket impl on top of FromIterator:
        impl<T,I> From<I> for $svec<T> where I:IntoIterator<Item=T> {
            fn from(iter:I) -> Self {
                Self::try_from_iter(iter).unwrap()
            }
        }

        impl<T,V> PartialEq<V> for $svec<T> where T:PartialEq, V:SVec<Item=T, Output=T> {
            fn eq(&self, other:&V) -> bool {
                if self.length!=other.len() { return false }
                for i in 0..self.length {
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
                for t in self.iter() {
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
                for t in self.iter() {
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

//pub struct Iter<'a,T:'a> {
//    svec: &'a dyn SVec<Item=T, Output=T>,
//    next_i: usize,
//}
//impl<'a,T> Iterator for Iter<'a,T> {
//    type Item = &'a T;
//    fn next(&mut self) -> Option<Self::Item> {
//        if self.next_i>=self.svec.len() { return None }
//        self.next_i+=1;
//        Some(&self.svec[self.next_i-1])
//    }
//}
//impl<T> Iter<'_,T> {
//    pub fn new<'a>(svec:&'a dyn SVec<Item=T, Output=T>) -> Iter<'a,T> {
//        Iter{svec:svec, next_i:0}
//    }
//}

//pub struct IterMut<'a,T:'a> {
//    svec: &'a mut (dyn SVec<Item=T, Output=T> + 'a),
//    next_i: usize,
//}
//impl<'a,T> Iterator for IterMut<'a,T> where T:'a {
//    type Item = &'a mut T;
//    fn next<'b>(&'b mut self) -> Option<&'a mut T> where 'a:'b {
//        if self.next_i>=self.svec.len() { return None }
//        self.next_i+=1;
//        Some(&mut self.svec[self.next_i-1])
//    }
//}
//impl<T> IterMut<'_,T> {
//    pub fn new<'a>(svec:&'a mut dyn SVec<Item=T, Output=T>) -> IterMut<'a,T> {
//        IterMut{svec:svec, next_i:0}
//    }
//}

