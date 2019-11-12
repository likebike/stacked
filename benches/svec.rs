// I really don't like the fact that '#!' is a syntax thing, often as the first bytes of a file.

#![feature(test)]
extern crate test;  // 'extern crate' seems to be required for this scenario: https://github.com/rust-lang/rust/issues/57288
use test::{Bencher, black_box};
use std::time::Instant;

use stacked::{self, SVec, SVec32};

#[bench]
fn vec1(b:&mut Bencher) {
    b.iter(|| {
        for _ in 1..100 {
            let mut v = Vec::<u8>::with_capacity(32);
            let cap = v.capacity();
            while v.len()<cap { v.push(b'1'); }

            black_box(v);
        }
    });
}

#[bench]
fn svec1(b:&mut Bencher) {
    let mut v = SVec32::<u8>::new();

    let start = Instant::now();
    for _ in 1..10_000_000 {
        v.clear();
        let cap = v.cap();
        while v.len()<cap { v.push(b'1').unwrap(); }
    }
    eprintln!("svec1 bench: {}",Instant::now().duration_since(start).as_secs_f64());

    b.iter(|| {
        let a = 333; black_box(a);
        for _ in 1..100 {
            v.clear();
            let cap = v.cap();
            while v.len()<cap { v.push(b'1').unwrap(); }

            black_box(&v);
        }
        let z = 444; black_box(z);
    });
}

