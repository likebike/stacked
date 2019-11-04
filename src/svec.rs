use kerr::KErr;

pub trait SVecBase<T> {
    fn new() -> Self;
    fn cap() -> usize where Self:Sized;  // https://doc.rust-lang.org/nightly/error-index.html#method-has-no-receiver
    fn len(&self) -> usize;

    // Not possible: (https://doc.rust-lang.org/nomicon/lifetime-mismatch.html)
    // fn push(&mut self, t:T) -> &T
    fn push(&self, t:T) -> Result<usize,KErr>;  // Append-only is corner-stone of safety model.
    fn get_copy(&self, i:usize) -> T where T:Copy;
    fn into_slice(&mut self) -> &mut [T];
}

pub trait SVec<T> : SVecBase<T> {
    fn get(&self, i:usize) -> &T;
    fn as_slice(&self) -> &[T];
}

pub trait SVecMut<T> : SVecBase<T> {
    fn pop(&self) -> T;
    fn update(&self, i:usize, t:T);
}

