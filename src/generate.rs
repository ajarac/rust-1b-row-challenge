use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use rand::Rng;

fn main() {
    let file = File::open("weather_stations.csv").expect("Failed to open weather_stations.csv");
    let reader = BufReader::new(file);

    let mut stations: Vec<(String, f64)> = Vec::new();

    for line in reader.lines().skip(1) {
        let line = line.unwrap();
        let parts: Vec<&str> = line.split(';').collect();
        if parts.len() == 2 {
            let name = parts[0].to_string();
            let mean_temp: f64 = parts[1].parse().unwrap_or(0.0);
            stations.push((name, mean_temp));
        }
    }

    let output = File::create("measurements.txt").expect("Failed to create measurements.txt");
    let mut writer = BufWriter::with_capacity(8 * 1024 * 1024, output);

    let mut rng = rand::rng();

    for _ in 0..10_000_000 {
        let (station, mean_temp) = &stations[rng.random_range(0..stations.len())];
        let temp = mean_temp + rng.random_range(-15.0..15.0);
        writeln!(writer, "{};{:.1}", station, temp).unwrap();
    }

    writer.flush().unwrap();
}