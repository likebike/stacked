// Avoid shebang.

#![feature(test)]
extern crate test;
use test::{Bencher, black_box};

use stacked::{SVec, SString32};

#[bench]
fn string1(b:&mut Bencher) {
    b.iter(|| {
        for _ in 1..100 {
            let mut s = String::with_capacity(32);
            let cap = s.capacity();
            while s.len()<cap { s.push('1'); }

            black_box(s);
        }
    });
}

#[bench]
fn sstring1(b:&mut Bencher) {
    b.iter(|| {
        let a = 333; black_box(a);
        for _ in 1..100 {
            let s = SString32::new();
            let cap = s.cap();
            while s.len()<cap { s.push(b'1').unwrap(); }

            black_box(s);
        }
        let z = 444; black_box(z);
    });
}

