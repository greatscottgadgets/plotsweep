use std::error::Error;
use std::collections::HashMap;

use chrono::naive::NaiveDateTime;
use image::{Rgb, RgbImage};
use imageproc::drawing::{draw_filled_rect_mut, draw_text_mut};
use imageproc::rect::Rect;
use plotters::{chart::ChartBuilder, prelude::{BitMapBackend, IntoDrawingArea}, style::{AsRelative, RGBColor, WHITE}};
use rusttype::{Font, Scale};
pub use scarlet::colormap::{ColorMap, ListedColorMap};

use super::csv::RecordCollection;

pub struct DrawSettings<'a> {
    pub colormap: &'a Box<dyn MyColorMap>,
    pub power_min: f32,
    pub power_max: f32,
    pub timestamps: bool,
}

pub trait MyColorMap {
    fn transform_single(&self, _: f64) -> scarlet::color::RGBColor;
}

impl MyColorMap for ListedColorMap {
    fn transform_single(&self, x: f64) -> scarlet::color::RGBColor {
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

fn build_lut(colormap: &Box<dyn MyColorMap>) -> Vec<plotters::style::RGBColor> {
    let mut lut = Vec::with_capacity(u16::MAX as usize + 1);
    for i in 0..u16::MAX as usize + 1 {
        let value = colormap.transform_single(i as f64 / u16::MAX as f64);
        lut.push(plotters::style::RGBColor{0: value.int_r(), 1: value.int_g(), 2: value.int_b()});
    }
    lut
}

pub fn draw_image(record_collection: &RecordCollection, output_path: &str, settings: &DrawSettings) -> Result<(), Box<dyn Error>> {
    let rc = record_collection; 

    let width = ((rc.freq_high - rc.freq_low) as f32 / rc.freq_step) as u32;
    let height = rc.timestamps.len() as u32;
    println!("Width: {} Height: {}", width, height);


    let b = BitMapBackend::new(output_path, (width, height))
        .into_drawing_area()
        .margin(20, 20, 20, 20);
    b.fill(&WHITE)?;
    let mut chart = ChartBuilder::on(&b)
        .margin(1)
        .caption("test", ("sans-serif", (10.percent_height())))
        .set_label_area_size(plotters::prelude::LabelAreaPosition::Left, (5i32).percent_width())
        .set_label_area_size(plotters::prelude::LabelAreaPosition::Top, (5i32).percent_height())
        .build_cartesian_2d(0..width, 0..height)?;
    chart.configure_mesh().draw()?;
    let pa = chart.plotting_area();

    let lut = build_lut(settings.colormap);
    let range = settings.power_max - settings.power_min;
    let scale = (lut.len()-1) as f32 / range;

    for record in &rc.records {
        let mut x = (((record.freq_low - rc.freq_low) as f32 / rc.freq_step) + 0.5) as u32;
        let y = rc.timestamps[&NaiveDateTime::new(record.date, record.time)];
        for sample in &record.samples {
            let scaled_pixel = (sample - settings.power_min) * scale;
            let value = &lut[(scaled_pixel as usize).clamp(0, lut.len()-1)];
            pa.draw_pixel((x, y), value)?;
            x += 1
        }
    }

    if settings.timestamps {
        // TODO: load a font properly
        let font = Vec::from(include_bytes!("/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf") as &[u8]);
        let font = Font::try_from_vec(font).unwrap();

        let height = 20.0;
        let scale = Scale{
            x: height * 1.5,
            y: height,
        };

        // Draw timestamps
        let mut timestamps = rc.timestamps.keys().collect::<Vec<_>>();
        timestamps.sort();
        for ts in timestamps.iter().skip(100).step_by(200) {
            let white = Rgb([255u8,255u8,255u8]);
            let black = Rgb([0u8, 0u8, 0u8]);
            let x = 10;
            let y = rc.timestamps[ts];
            let text = &ts.to_string();
            let h = scale.y as u32;
            //draw_filled_rect_mut(&mut img, Rect::at(0, y as i32).of_size(20, 2), white);
            //draw_text_mut(&mut img, black, x+2, y-h+2, scale, &font, text);
            //draw_text_mut(&mut img, white, x, y-h, scale, &font, text);
        }
    }
    //img.save(output_path)?;
    Ok(())
}
