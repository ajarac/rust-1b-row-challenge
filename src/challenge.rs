use std::fs::File;
use std::time::Instant;
use memmap2::Mmap;
use rayon::prelude::*;
use rustc_hash::FxHashMap;

struct Statistic {
    min: i32,
    sum: i64,
    max: i32,
    counter: usize,
}

impl Statistic {
    pub fn new() -> Self {
        Self {
            min: i32::MAX,
            sum: 0,
            max: i32::MIN,
            counter: 0,
        }
    }

    pub fn add(&mut self, value: i32) {
        if value < self.min {
            self.min = value;
        }
        if value > self.max {
            self.max = value;
        }
        self.sum += value as i64;
        self.counter += 1;
    }

    pub fn merge(&mut self, other: &Statistic) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
        self.sum += other.sum;
        self.counter += other.counter;
    }
}

fn parse_temperature(bytes: &[u8]) -> i32 {
    let mut value = 0i32;
    let mut negative = false;
    let mut i = 0;

    if bytes[0] == b'-' {
        negative = true;
        i = 1;
    }

    // Parse digits before decimal
    while i < bytes.len() && bytes[i] != b'.' {
        value = value * 10 + (bytes[i] - b'0') as i32;
        i += 1;
    }

    // Skip '.'
    i += 1;

    // Parse single digit after decimal
    if i < bytes.len() {
        value = value * 10 + (bytes[i] - b'0') as i32;
    }

    if negative {
        -value
    } else {
        value
    }
}

fn main() {
    let start = Instant::now();

    let filename = "measurements.txt";
    let file = File::open(filename).expect("file not found");
    let mmap = unsafe { Mmap::map(&file).expect("failed to mmap file") };

    let num_threads = rayon::current_num_threads();
    let chunk_size = mmap.len() / num_threads;

    let results: Vec<FxHashMap<Box<str>, Statistic>> = (0..num_threads)
        .into_par_iter()
        .map(|thread_id| {
            let mut start = thread_id * chunk_size;

            if start != 0 {
                while start < mmap.len() && mmap[start] != b'\n' {
                    start += 1;
                }
                start += 1;
            }

            let end = if thread_id == num_threads - 1 {
                mmap.len()
            } else {
                let mut e = (thread_id + 1) * chunk_size;
                while e < mmap.len() && mmap[e] != b'\n' {
                    e += 1;
                }
                e + 1
            };

            let mut local_map: FxHashMap<Box<str>, Statistic> = FxHashMap::default();
            let data = &mmap[start..end];
            let mut line_start = 0;

            for i in 0..data.len() {
                if data[i] == b'\n' {
                    let line = &data[line_start..i];
                    line_start = i + 1;

                    if let Some(semicolon_pos) = line.iter().position(|&b| b == b';') {
                        let station = std::str::from_utf8(&line[..semicolon_pos]).unwrap();
                        let temp_bytes = &line[semicolon_pos + 1..];
                        let value = parse_temperature(temp_bytes);

                        let statistic = local_map
                            .entry(station.into())
                            .or_insert_with(Statistic::new);

                        statistic.add(value);
                    }
                }
            }

            local_map
        })
        .collect();

    let mut hash_map: FxHashMap<Box<str>, Statistic> = FxHashMap::default();
    for local_map in results {
        for (station, stats) in local_map {
            hash_map
                .entry(station)
                .and_modify(|existing| existing.merge(&stats))
                .or_insert(stats);
        }
    }

    let mut sorted: Vec<_> = hash_map.into_iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    for (station, stat) in sorted {
        println!(
            "{};{:.1}/{:.1}/{:.1}",
            station,
            stat.min as f64 / 10.0,
            (stat.sum as f64 / stat.counter as f64) / 10.0,
            stat.max as f64 / 10.0
        );
    }

    let duration = start.elapsed();
    println!("Time: {:.3}s", duration.as_millis() as f64 / 1000.0);
}