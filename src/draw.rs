use std::error::Error;

use chrono::naive::NaiveDateTime;
use image::{Rgb, RgbImage};

use super::csv::RecordCollection;

#[derive(Debug)]
pub struct DrawSettings {
    pub power_min: f32,
    pub power_max: f32,
}

pub fn draw_image(record_collection: &RecordCollection, output_path: &str, settings: &DrawSettings) -> Result<(), Box<dyn Error>> {
    let rc = record_collection; 

    let width = ((rc.freq_high - rc.freq_low) as f32 / rc.freq_step) as u32;
    let height = rc.timestamps.len() as u32;
    println!("Width: {} Height: {}", width, height);

    let mut img = RgbImage::new(width, height);

    let range = settings.power_max - settings.power_min;
    let scale = 1f32 / range;
    for record in &rc.records {
        let mut x = ((record.freq_low - rc.freq_low) as f32 / rc.freq_step) as u32;
        let y = rc.timestamps[&NaiveDateTime::new(record.date, record.time)];
        for sample in &record.samples {
            let scaled_pixel = (sample - settings.power_min) * scale;
            let value = (scaled_pixel.clamp(0f32, 1f32) * 255f32) as u8;
            img.put_pixel(x, y, Rgb([value, 0, 0]));
            x += 1
        }
    }
    img.save(output_path)?;
    Ok(())
}
