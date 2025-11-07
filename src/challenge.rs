use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
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
    let file = BufReader::new(file);

    let mut hash_map: HashMap<String, Statistic> = HashMap::new();

    for line in file.lines() {
        let line = line.expect("error reading file");
        let parts: Vec<&str> = line.split(';').collect();
        if parts.len() != 2 {
            continue;
        }

        let station: String = parts[0].to_string(); // owned
        let value: f64 = parts[1].parse().unwrap_or(0.0);

        let statistic = hash_map
            .entry(station) // moves ownership into the map
            .or_insert_with(Statistic::new);

        statistic.add(value);
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
    println!("Time: {:.2}s", duration.as_secs_f64());
}
