use std::{collections::HashMap, fs::File, thread};

use crate::reader::{OrphanCollector, Reader};

pub fn custom_single() {
    let file = File::open("measurements.txt").expect("could not open file");
    let orphan_collector = OrphanCollector::new();
    let mut lines = 0;

    {
        let mut reader = Reader::new(file, &orphan_collector).unwrap();

        while let Some(_line) = reader.read_line() {
            lines += 1;
        }
    }

    let orphans = orphan_collector.get_orphans();

    lines += orphans.len();

    println!("lines: {lines}");
}

pub fn custom_multi() {
    let jobs = 12;
    let file = File::open("measurements.txt").expect("could not open file");
    let orphan_collector = OrphanCollector::new();

    let mut handles = Vec::with_capacity(jobs);
    for _ in 0..jobs {
        let mut reader = Reader::new(file.try_clone().unwrap(), &orphan_collector).unwrap();

        handles.push(thread::spawn(move || {
            let mut lines = 0;
            while let Some(_line) = reader.read_line() {
                lines += 1;
            }

            lines
        }));
    }

    let mut lines = 0;
    for handle in handles {
        let thread_lines = handle.join().unwrap();
        lines += thread_lines;
    }

    let orphans = orphan_collector.get_orphans();

    lines += orphans.len();

    println!("lines: {lines}");
}
