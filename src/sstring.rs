use kerr::KErr;

pub trait SString {
    fn cap() -> usize where Self:Sized;  // https://doc.rust-lang.org/nightly/error-index.html#method-has-no-receiver
    fn len(&self) -> usize;

    fn push(&self, b:u8) -> Result<usize,KErr>;
}

