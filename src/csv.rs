use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fs::File;

use chrono::naive::{NaiveDateTime, NaiveDate, NaiveTime};
use serde::Deserialize;

#[derive(Default, Debug)]
pub struct RecordCollection {
    pub records: Vec<Record>,
    pub timestamps: HashMap<NaiveDateTime, u32>,
    pub freq_low: u64,
    pub freq_high: u64,
    pub freq_step: f32,
}

#[derive(Debug, Deserialize)]
pub struct Record {
    #[serde(with = "custom_date")]
    pub date: NaiveDate,
    #[serde(with = "custom_time")]
    pub time: NaiveTime,
    pub freq_low: u64,
    pub freq_high: u64,
    pub freq_step: f32,
    pub num_samples: u32,
    pub samples: Vec<f32>,
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

pub fn load_records(input_path: &str) -> Result<RecordCollection, Box<dyn Error>> {
    let input = File::open(input_path)?;
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_reader(input);

    let mut rc = RecordCollection{ ..Default::default() };

    // Loop through all lines & parse records
    // also keep track of frequency range & unique timestamps to determine final image size
    println!("Loading records...");
    let mut step: Option<f32> = None;
    let mut timestamps = HashSet::new();
    for result in rdr.deserialize() {
        // Break out on error and try to continue processing, for cases where the CSV is still being written
        if let Err(e) = result {
            println!("Warning: {}", e);
            break;
        }
        let record: Record = result?;
        rc.freq_low = std::cmp::min(rc.freq_low, record.freq_low);
        rc.freq_high = std::cmp::max(rc.freq_high, record.freq_high);
        if let Some(s) = step {
            if s != record.freq_step {
                return Err("Frequency step must be constant".into());
            }
        } else {
            step = Some(record.freq_step);
        }
        timestamps.insert(NaiveDateTime::new(record.date, record.time));
        rc.records.push(record);
    }
    rc.freq_step = step.unwrap();

    // Sort the timestamp set & produce a map from timestamp -> row number
    let mut timestamps: Vec<_> = timestamps.iter().collect();
    timestamps.sort_unstable();
    rc.timestamps = timestamps.iter().enumerate().map(|(i, &x)| (*x, i as u32)).collect();

    Ok(rc)
}

