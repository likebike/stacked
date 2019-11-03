use kerr::KErr;

pub trait SVec<T> {
    fn cap() -> usize where Self:Sized;  // https://doc.rust-lang.org/nightly/error-index.html#method-has-no-receiver
    fn len(&self) -> usize;

    // Not possible: (https://doc.rust-lang.org/nomicon/lifetime-mismatch.html)
    // fn push(&mut self, t:T) -> &T
    fn push(&self, t:T) -> Result<usize,KErr>;  // Append-only is corner-stone of safety model.
    fn get(&self, i:usize) -> &T;
}

