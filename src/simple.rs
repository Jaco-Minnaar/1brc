use std::{
    fs::File,
    io::{BufReader, Read},
    thread,
    time::Instant,
};

use kanal::{Receiver, Sender};

const CALC_JOBS: usize = 10;

pub fn simple() {
    let file = File::open("measurements.txt").unwrap();
    // let (point_tx, point_rx) = kanal::bounded(CALC_JOBS - 1);
    let (line_tx, line_rx) = kanal::unbounded();
    let mut handles = Vec::new();

    for _ in 0..CALC_JOBS {
        let reader = BufReader::with_capacity(1024 * 1024 * 20, file.try_clone().unwrap());
        let line_tx = line_tx.clone();
        let read_handle = thread::spawn(move || {
            read(reader, line_tx);
        });
        handles.push(read_handle);
    }

    drop(line_tx);

    let mut lines = 0;
    for line_count in line_rx {
        lines += line_count;
    }

    println!("lines: {lines}");

    // let mut calc_txs = Vec::with_capacity(CALC_JOBS);
    //
    // for _ in 0..CALC_JOBS {
    //     let (tx, rx) = kanal::bounded(0);
    //     calc_txs.push(tx);
    //     let point_tx = point_tx.clone();
    //
    //     handles.push(thread::spawn(move || {
    //         calc(rx, point_tx);
    //     }))
    // }
    // drop(point_tx);
    //
    // let calc_handle = thread::spawn(move || {
    //     let mut worker_idx = 0;
    //     for lines in line_rx {
    //         calc_txs[worker_idx].send(lines).unwrap();
    //
    //         worker_idx = (worker_idx + 1) % CALC_JOBS;
    //     }
    // });
    // handles.push(calc_handle);
    //
    // let mut tree = Tree::new();
    //
    // let mut point_no = 0;
    // for points in point_rx {
    //     for (point, city) in points {
    //         tree.update(city, point);
    //         point_no += 1;
    //     }
    // }
    //
    // println!("Valid points: {point_no}");

    for handle in handles {
        handle.join().unwrap();
    }
}

const LINE_BUFFER_SIZE: usize = 100_000;

// fn read(mut reader: BufReader<File>, line_tx: Sender<Vec<[u8; 50]>>) {
fn read(mut reader: BufReader<File>, line_tx: Sender<usize>) {
    let mut line_count = 0;
    // let mut lines = Vec::with_capacity(LINE_BUFFER_SIZE);
    let mut before = Instant::now();
    let mut longest = 0;
    for byte in reader.bytes() {
        let byte = byte.unwrap();

        if byte == 0xA {
            line_count += 1;

            if line_count >= LINE_BUFFER_SIZE {
                line_tx.send(line_count).unwrap();
                line_count = 0;
            }
        }
    }

    line_tx.send(line_count).unwrap();
}

fn calc(line_rx: Receiver<Vec<([u8; 50], usize)>>, point_tx: Sender<Vec<(f32, [u8; 32])>>) {
    let mut points = Vec::with_capacity(100_000);

    let mut longest = 0;
    let mut before = Instant::now();
    for lines in line_rx {
        for (buffer, length) in lines {
            if let Some(point) = parse(&buffer[0..length]) {
                points.push(point);
            }
        }

        point_tx.send(points.clone()).unwrap();
        points.clear();
    }
}

fn parse(bytes: &[u8]) -> Option<(f32, [u8; 32])> {
    let mut parts = bytes.split(|b| *b == 0x3B);
    let name = parts.next()?;
    let temp = parts.next()?;

    let num_str = String::from_utf8_lossy(temp);
    let num = num_str.parse::<f32>().ok()?;

    if name.len() > 32 {
        return None;
    }

    let mut bytes = [0; 32];

    for (i, byte) in name.iter().enumerate() {
        bytes[i] = *byte;
    }

    Some((num, bytes))
}
