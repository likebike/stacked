use kerr::KErr;

pub trait SVec<T> {
    // ---- Append-Only Interface ----
    // If you ONLY use this section, you can't have bugs.
    fn new() -> Self where Self:Sized;  // https://doc.rust-lang.org/nightly/error-index.html#method-has-no-receiver
    fn cap() -> usize where Self:Sized;
    fn len(&self) -> usize;
    fn push(&self, t:T) -> Result<usize,KErr>;
    fn get(&self, i:usize) -> &T;
    fn get_copy(&self, i:usize) -> T where T:Copy;
    fn as_slice(&self) -> &[T];
    //fn iter(&self) -> impl Iterator<Item=&T>;  // Currently not able to return 'impl Trait' from Trait methods.

    // ---- Mutation Interface ----
    // If you use any of this section AT ALL, it is up to you to keep the bugs out.
    fn pop(&mut self) -> T;
    fn set(&mut self, i:usize, t:T);
    fn insert(&mut self, i:usize, t:T);
    fn remove(&mut self, i:usize) -> T;
    fn as_slice_mut(&mut self) -> &mut [T];
    //fn iter_mut(&mut self) -> impl Iterator<Item=&mut T>  // Currently not able to return 'impl Trait' from Trait methods.
}

