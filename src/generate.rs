use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use rand::Rng;
use std::time::Instant;

fn main() {
    let start = Instant::now();

    let file = File::open("weather_stations.csv").expect("Failed to open weather_stations.csv");
    let reader = BufReader::new(file);

    let stations: Vec<(String, f64)> = reader
        .lines()
        .skip(1)
        .filter_map(|line| {
            let line = line.ok()?;
            let mut parts = line.split(';');
            let name = parts.next()?.to_string();
            let mean_temp = parts.next()?.parse().ok()?;
            Some((name, mean_temp))
        })
        .collect();

    let output = File::create("measurements.txt").expect("Failed to create measurements.txt");
    let mut writer = BufWriter::with_capacity(64 * 1024 * 1024, output);

    let mut rng = rand::rng();
    let mut buffer = String::with_capacity(128);

    const TOTAL: u64 = 1_000_000_000;
    const UPDATE_INTERVAL: u64 = TOTAL / 100;

    for i in 0..TOTAL {
        if i % UPDATE_INTERVAL == 0 {
            print!("\r{}%", i / (TOTAL / 100));
            std::io::stdout().flush().unwrap();
        }

        let idx = rng.random_range(0..stations.len());
        let (station, mean_temp) = &stations[idx];
        let temp = mean_temp + rng.random_range(-15.0..15.0);

        buffer.clear();
        use std::fmt::Write as FmtWrite;
        let _ = write!(&mut buffer, "{};{:.1}\n", station, temp);
        writer.write_all(buffer.as_bytes()).unwrap();
    }

    writer.flush().unwrap();
    println!("\r100%");
    println!("Generated in {:?}", start.elapsed());
}