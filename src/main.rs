use std::error::Error;
use std::process;

use chrono::naive::NaiveDateTime;
use clap::{Arg, App, value_t};
use image::{Rgb, RgbImage};

mod csv;

fn heatmap(input_path: &str, output_path: &str, power_min: f32, power_max: f32) -> Result<(), Box<dyn Error>> {
    let rc = csv::load_records(input_path)?;

    let width = ((rc.freq_high - rc.freq_low) as f32 / rc.freq_step) as u32;
    let height = rc.timestamps.len() as u32;
    println!("Width: {} Height: {}", width, height);

    let mut img = RgbImage::new(width, height);

    let range = power_max - power_min;
    let scale = 1f32 / range;
    for record in rc.records {
        let mut x = ((record.freq_low - rc.freq_low) as f32 / rc.freq_step) as u32;
        let y = rc.timestamps[&NaiveDateTime::new(record.date, record.time)];
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
        .arg(Arg::with_name("INPUT")
             .required(true))
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

    let input_path = matches.value_of("INPUT").unwrap();
    let output_path = matches.value_of("OUTPUT").unwrap();
    let power_min = value_t!(matches, "power-min", f32).unwrap_or_else(|e| e.exit());
    let power_max = value_t!(matches, "power-max", f32).unwrap_or_else(|e| e.exit());

    if let Err(err) = heatmap(input_path, output_path, power_min, power_max) {
        println!("error running heatmap: {}", err);
        process::exit(1);
    }
}
