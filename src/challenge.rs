use memmap2::Mmap;
use std::collections::HashMap;
use std::fs::File;
use std::str::from_utf8;
use std::time::Instant;

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
}

fn main() {
    let start = Instant::now();

    let filename = "measurements.txt";
    let file = File::open(filename).expect("file not found");
    let mmap = unsafe { Mmap::map(&file).expect("failed to mmap file") };

    let mut hash_map: HashMap<String, Statistic> = HashMap::new();

    let data = &mmap[..];
    let mut line_start = 0;

    for i in 0..data.len() {
        if data[i] == b'\n' {
            let line = &data[line_start..i];
            line_start = i + 1;

            if let Some(semicolon_pos) = line.iter().position(|&b| b == b';') {
                let station = from_utf8(&line[..semicolon_pos]).unwrap();
                let temp_str = from_utf8(&line[semicolon_pos + 1..]).unwrap();
                let value: f64 = temp_str.parse().unwrap_or(0.0);

                let statistic = hash_map
                    .entry(station.to_string())
                    .or_insert_with(Statistic::new);

                statistic.add(value);
            }
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
