use core::panic;
use std::fs::File;
use std::io::{Read, Result};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use kanal::Sender;

const LINE_LEN: usize = 64;
const BUFFER_SIZE: usize = 1024 * 1024 * 32;

type Line = [u8; LINE_LEN];

struct TrackingLine {
    line: Line,
    idx: usize,
}

impl TrackingLine {
    fn new() -> Self {
        Self {
            line: [0; LINE_LEN],
            idx: 0,
        }
    }

    fn add(&mut self, val: u8) {
        if self.idx >= LINE_LEN {
            panic!("line longer than {LINE_LEN} bytes");
        }
        self.line[self.idx] = val;
        self.idx += 1;
    }
}

pub struct Reader {
    file: File,
    buffer: Vec<u8>,
    buffer_size: usize,
    buffer_num: usize,
    idx: usize,
    block: Arc<AtomicUsize>,
    orphan_tx: Sender<(usize, [[u8; LINE_LEN]; 2])>,
    start_orphan: Option<[u8; LINE_LEN]>,
    current_line: TrackingLine,
}

impl Reader {
    pub fn new(file: File, orphan_collector: &OrphanCollector) -> Result<Self> {
        Ok(Self {
            file,
            buffer: vec![0; BUFFER_SIZE],
            buffer_size: BUFFER_SIZE,
            buffer_num: 0,
            idx: BUFFER_SIZE,
            block: orphan_collector.get_block(),
            orphan_tx: orphan_collector.get_sender(),
            start_orphan: None,
            current_line: TrackingLine::new(),
        })
    }

    pub fn read_line(&mut self) -> Option<Line> {
        loop {
            if self.idx >= self.buffer_size {
                self.refresh_buffer();
            }

            let byte = self.buffer.get(self.idx).unwrap();
            self.idx += 1;

            if byte == &0xA {
                self.current_line = TrackingLine::new();

                return Some(self.current_line.line);
            } else if byte == &0 {
                self.end_buffer();

                return None;
            } else {
                self.current_line.add(*byte);
            }
        }
    }

    fn end_buffer(&mut self) {
        if self.start_orphan.is_some() {
            self.orphan_tx
                .send((
                    self.buffer_num,
                    [self.start_orphan.unwrap(), self.current_line.line],
                ))
                .unwrap();
        }
    }

    fn refresh_buffer(&mut self) {
        self.end_buffer();
        self.read_to_buffer();
        self.idx = 0;

        let mut orphan = [0; LINE_LEN];
        let mut idx = 0;
        loop {
            let byte = *self.buffer.get(self.idx).unwrap();
            self.idx += 1;
            if byte == 0xA || byte == 0 {
                break;
            }

            orphan[idx] = byte;
            idx += 1;
        }

        self.start_orphan.replace(orphan);
        self.current_line = TrackingLine::new();
    }

    fn read_to_buffer(&mut self) {
        self.buffer.fill(0);
        let num_bytes = self.file.read(self.buffer.as_mut_slice()).unwrap();
        self.buffer_num = self.block.fetch_add(1, Ordering::SeqCst) + 1;
        self.buffer_size = num_bytes;
    }
}

pub struct OrphanCollector {
    tx: Option<Sender<(usize, [Line; 2])>>,
    block_num: Arc<AtomicUsize>,
    collector_handle: JoinHandle<Vec<[Line; 2]>>,
}

impl OrphanCollector {
    pub fn new() -> Self {
        let (tx, rx) = kanal::unbounded::<(usize, [Line; 2])>();

        let handle = thread::spawn(move || {
            let mut orphans =
                vec![[[0 as u8; LINE_LEN]; 2]; (36 * 1024 * 1024 * 1024) / BUFFER_SIZE];
            for (block, orphan) in rx {
                orphans[block] = orphan;
            }

            return orphans;
        });

        Self {
            tx: Some(tx),
            block_num: Arc::new(AtomicUsize::new(0)),
            collector_handle: handle,
        }
    }

    fn get_block(&self) -> Arc<AtomicUsize> {
        Arc::clone(&self.block_num)
    }

    fn get_sender(&self) -> Sender<(usize, [Line; 2])> {
        if let Some(tx) = &self.tx {
            tx.clone()
        } else {
            panic!("tried to get new sender after sender was dropped");
        }
    }

    pub fn get_orphans(mut self) -> Vec<Line> {
        let Some(tx) = self.tx.take() else {
            panic!("OrphanCollector does not have a Sender (tx)");
        };

        drop(tx);

        let orphaned_lines = self.collector_handle.join().unwrap();
        let mut orphans = vec![];
        for i in 0..orphaned_lines.len() - 1 {
            let mut line = TrackingLine::new();
            for byte in orphaned_lines[i][1] {
                if byte == 0 {
                    break;
                }

                line.add(byte);
            }

            for byte in orphaned_lines[i + 1][0] {
                if byte == 0 {
                    break;
                }

                line.add(byte);
            }

            if line.idx > 0 {
                orphans.push(line.line);
            }
        }

        // for orphan in &orphans {
        //     dbg!(String::from_utf8_lossy(orphan));
        // }

        return orphans;
    }
}
