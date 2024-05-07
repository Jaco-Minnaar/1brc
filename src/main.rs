#![allow(clippy::pedantic)]
#![allow(clippy::perf)]

use std::time::Instant;

mod custom;
mod reader;
mod simple;
mod tree;

fn main() {
    let before = Instant::now();
    // simple::simple();
    custom::custom_multi();

    println!("Elapsed time: {}ms", before.elapsed().as_millis());
}
