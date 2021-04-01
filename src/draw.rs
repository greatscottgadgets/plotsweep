use std::error::Error;
use std::collections::HashMap;

use chrono::naive::NaiveDateTime;
use image::{Rgb, RgbImage};
pub use scarlet::colormap::{ColorMap, ListedColorMap};
use scarlet::color::RGBColor;

use super::csv::RecordCollection;

pub struct DrawSettings<'a> {
    pub colormap: &'a Box<dyn MyColorMap>,
    pub power_min: f32,
    pub power_max: f32,
}

pub trait MyColorMap {
    fn transform_single(&self, _: f64) -> RGBColor;
}

impl MyColorMap for ListedColorMap {
    fn transform_single(&self, x: f64) -> RGBColor {
        scarlet::colormap::ColorMap::transform_single(self, x)
    }
}

pub fn colormaps() -> HashMap<&'static str, Box<dyn MyColorMap>> {
    let mut maps: HashMap<&'static str, Box<dyn MyColorMap>> = HashMap::new();
    maps.insert("viridis", Box::new(ListedColorMap::viridis()));
    maps.insert("magma",   Box::new(ListedColorMap::magma()));
    maps.insert("inferno", Box::new(ListedColorMap::inferno()));
    maps.insert("plasma",  Box::new(ListedColorMap::plasma()));
    maps
}

fn build_lut(colormap: &Box<dyn MyColorMap>) -> Vec<RGBColor> {
    let mut lut = Vec::with_capacity(u16::MAX as usize + 1);
    for i in 0..u16::MAX as usize + 1 {
        lut.push(colormap.transform_single(i as f64 / u16::MAX as f64));
    }
    lut
}

pub fn draw_image(record_collection: &RecordCollection, output_path: &str, settings: &DrawSettings) -> Result<(), Box<dyn Error>> {
    let rc = record_collection; 

    let width = ((rc.freq_high - rc.freq_low) as f32 / rc.freq_step) as u32;
    let height = rc.timestamps.len() as u32;
    println!("Width: {} Height: {}", width, height);

    let mut img = RgbImage::new(width, height);

    let lut = build_lut(settings.colormap);
    let range = settings.power_max - settings.power_min;
    let scale = (lut.len()-1) as f32 / range;
    for record in &rc.records {
        let mut x = ((record.freq_low - rc.freq_low) as f32 / rc.freq_step) as u32;
        let y = rc.timestamps[&NaiveDateTime::new(record.date, record.time)];
        for sample in &record.samples {
            let scaled_pixel = (sample - settings.power_min) * scale;
            let value = lut[(scaled_pixel as usize).clamp(0, lut.len()-1)];
            img.put_pixel(x, y, Rgb([value.int_r(), value.int_g(), value.int_b()]));
            x += 1
        }
    }
    img.save(output_path)?;
    Ok(())
}
