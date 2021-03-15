use std::error::Error;
use std::io;
use std::process;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Record {
    date: String,
    time: String,
    freq_low: u64,
    freq_high: u64,
    freq_step: f32,
    num_samples: u32,
    samples: Vec<f32>,
}

fn example() -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_reader(io::stdin());
    for result in rdr.deserialize() {
        // Notice that we need to provide a type hint for automatic
        // deserialization.
        let record: Record = result?;
        println!("{:?}", record);
    }
    Ok(())
}

fn main() {
    if let Err(err) = example() {
        println!("error running example: {}", err);
        process::exit(1);
    }
}
