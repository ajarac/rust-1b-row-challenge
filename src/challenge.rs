use std::collections::HashMap;
use std::fs::File;
use std::time::Instant;
use memmap2::Mmap;
use rayon::prelude::*;

struct Statistic {
    min: f64,
    sum: f64,
    max: f64,
    counter: usize,
}

impl Statistic {
    pub fn new() -> Self {
        Self {
            min: 100.0,
            sum: 0.0,
            max: -100.0,
            counter: 0,
        }
    }

    pub fn add(&mut self, value: f64) {
        if value < self.min {
            self.min = value;
        }
        if value > self.max {
            self.max = value;
        }
        self.sum += value;
        self.counter += 1;
    }

    pub fn merge(&mut self, other: &Statistic) {
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
        self.sum += other.sum;
        self.counter += other.counter;
    }
}

fn main() {
    let start = Instant::now();

    let filename = "measurements.txt";
    let file = File::open(filename).expect("file not found");
    let mmap = unsafe { Mmap::map(&file).expect("failed to mmap file") };

    let num_threads = rayon::current_num_threads();
    let chunk_size = mmap.len() / num_threads;

    let results: Vec<HashMap<String, Statistic>> = (0..num_threads)
        .into_par_iter()
        .map(|thread_id| {
            let mut start = thread_id * chunk_size;

            // Align to next newline (except for first thread)
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

            let mut local_map: HashMap<String, Statistic> = HashMap::new();
            let data = &mmap[start..end];
            let mut line_start = 0;

            for i in 0..data.len() {
                if data[i] == b'\n' {
                    let line = &data[line_start..i];
                    line_start = i + 1;

                    if let Some(semicolon_pos) = line.iter().position(|&b| b == b';') {
                        let station = std::str::from_utf8(&line[..semicolon_pos]).unwrap();
                        let temp_str = std::str::from_utf8(&line[semicolon_pos + 1..]).unwrap();
                        let value: f64 = temp_str.parse().unwrap_or(0.0);

                        let statistic = local_map
                            .entry(station.to_string())
                            .or_insert_with(Statistic::new);

                        statistic.add(value);
                    }
                }
            }

            local_map
        })
        .collect();

    // Merge all local maps
    let mut hash_map: HashMap<String, Statistic> = HashMap::new();
    for local_map in results {
        for (station, stats) in local_map {
            hash_map
                .entry(station)
                .and_modify(|existing| existing.merge(&stats))
                .or_insert(stats);
        }
    }

    let mut sorted: Vec<_> = hash_map.into_iter().collect();
    sorted.sort_by_key(|a| a.0.clone());

    for (station, stat) in sorted {
        println!(
            "{};{:.1}/{:.1}/{:.1}",
            station,
            stat.min,
            stat.sum / stat.counter as f64,
            stat.max
        );
    }

    let duration = start.elapsed();
    println!("Time: {:.3}s", duration.as_millis() as f64 / 1000.0);
}