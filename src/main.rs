use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io;
use std::process;

use chrono::naive::{NaiveDateTime, NaiveDate, NaiveTime};
use clap::{Arg, App, value_t};
use image::{Rgb, RgbImage};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(with = "custom_date")]
    date: NaiveDate,
    #[serde(with = "custom_time")]
    time: NaiveTime,
    freq_low: u64,
    freq_high: u64,
    freq_step: f32,
    num_samples: u32,
    samples: Vec<f32>,
}

mod custom_date {
    use chrono::naive::NaiveDate;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        Ok(NaiveDate::parse_from_str(&s, "%Y-%m-%d").unwrap())
    }
}

mod custom_time {
    use chrono::naive::NaiveTime;
    use serde::{Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<NaiveTime, D::Error>
    where D: Deserializer<'de> {
        let s = String::deserialize(deserializer)?;
        Ok(NaiveTime::parse_from_str(&s, "%H:%M:%S%.f").unwrap())
    }
}

fn heatmap(output_path: &str, power_min: f32, power_max: f32) -> Result<(), Box<dyn Error>> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_reader(io::stdin());

    // Loop through all lines & parse records
    // also keep track of frequency range & unique timestamps to determine final image size
    println!("Loading records...");
    let mut records = Vec::<Record>::new();
    let mut min = std::u64::MAX;
    let mut max = std::u64::MIN;
    let mut step: Option<f32> = None;
    let mut timestamps = HashSet::new();
    for result in rdr.deserialize() {
        // Break out on error and try to continue processing, for cases where the CSV is still being written
        if let Err(e) = result {
            println!("Warning: {}", e);
            break;
        }
        let record: Record = result?;
        min = std::cmp::min(min, record.freq_low);
        max = std::cmp::max(max, record.freq_high);
        if let Some(s) = step {
            if s != record.freq_step {
                return Err("Frequency step must be constant".into());
            }
        } else {
            step = Some(record.freq_step);
        }
        timestamps.insert(NaiveDateTime::new(record.date, record.time));
        records.push(record);
    }
    let step = step.unwrap();

    // Sort the timestamp set & produce a map from timestamp -> row number
    let mut timestamps: Vec<_> = timestamps.iter().collect();
    timestamps.sort_unstable();
    let timestamps: HashMap<_, _> = timestamps.iter().enumerate().map(|(i, x)| (*x, i as u32)).collect();

    let width = ((max - min) as f32 / step) as u32;
    let height = timestamps.len() as u32;
    println!("Width: {} Height: {}", width, height);

    let mut img = RgbImage::new(width, height);

    for record in records {
        let mut x = ((record.freq_low - min) as f32 / step) as u32;
        let y = timestamps[&NaiveDateTime::new(record.date, record.time)];
        let range = power_max - power_min;
        let scale = 1f32 / range;
        for sample in record.samples {
            let scaled_pixel = (sample - power_min) * scale;
            let value = (scaled_pixel.clamp(0f32, 1f32) * 255f32) as u8;
            img.put_pixel(x, y, Rgb([value, 0, 0]));
            x += 1
        }
    }
    img.save(output_path)?;
    Ok(())
}

fn main() {
    let matches = App::new("heatmap")
        .arg(Arg::with_name("OUTPUT")
             .required(true))
        .arg(Arg::with_name("power-min")
             .long("power-min")
             .takes_value(true)
             .allow_hyphen_values(true)
             .default_value("-70"))
        .arg(Arg::with_name("power-max")
             .long("power-max")
             .takes_value(true)
             .allow_hyphen_values(true)
             .default_value("-30"))
        .get_matches();

    let output_path = matches.value_of("OUTPUT").unwrap();
    let power_min = value_t!(matches, "power-min", f32).unwrap_or_else(|e| e.exit());
    let power_max = value_t!(matches, "power-max", f32).unwrap_or_else(|e| e.exit());

    if let Err(err) = heatmap(output_path, power_min, power_max) {
        println!("error running heatmap: {}", err);
        process::exit(1);
    }
}
