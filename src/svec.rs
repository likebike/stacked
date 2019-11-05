use kerr::KErr;

// If you only use the append-only (&self) interface, then you can't have bugs.
// If you use the mutation (&mut self) interface, then it's up to you to keep the bugs away.
pub trait SVec<T> {
    // ---- Append-Only Interface ----
    fn new() -> Self where Self:Sized;  // https://doc.rust-lang.org/nightly/error-index.html#method-has-no-receiver
    fn cap() -> usize where Self:Sized;
    fn len(&self) -> usize;
    fn push(&self, t:T) -> Result<usize,KErr>;  // Append is always safe.  Don't require exclusive access.
    fn get(&self, i:usize) -> &T;
    fn get_copy(&self, i:usize) -> T where T:Copy;
    fn as_slice(&self) -> &[T];

    // ---- Mutation Interface ----
    fn pop(&mut self) -> T;
    fn set(&mut self, i:usize, t:T);
    fn insert(&mut self, i:usize, t:T);
    fn remove(&mut self, i:usize) -> T;
}

