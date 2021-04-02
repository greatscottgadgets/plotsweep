use std::error::Error;
use std::collections::HashMap;

use chrono::naive::NaiveDateTime;
use super::coord::IntoReversedAxis;
use plotters::{chart::ChartBuilder, prelude::{BitMapBackend, IntoDrawingArea, LabelAreaPosition, RangedDateTime}, style::{WHITE}};
pub use scarlet::colormap::{ColorMap, ListedColorMap};

use super::csv::RecordCollection;

pub struct DrawSettings<'a> {
    pub colormap: &'a Box<dyn MyColorMap>,
    pub power_min: f32,
    pub power_max: f32,
    pub hide_axes: bool,
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

    let margins = if settings.hide_axes {
        (0, 0)
    } else {
        (150, 40)
    };

    let b = BitMapBackend::new(output_path, (width+margins.0*2, height+margins.1*2))
        .into_drawing_area();
    b.fill(&WHITE)?;

    let plot_area = if settings.hide_axes {
        b
    } else {
        // Build the y-axis by getting the min/max timestamps.
        //
        // By default, the latest time will be drawn at the top,
        // so we reverse the axis to get a normal waterfall-style plot.
        let ts_min = rc.timestamps.keys().min().unwrap();
        let ts_max = rc.timestamps.keys().max().unwrap();
        let ts_range = *ts_min..*ts_max;
        let y_axis = RangedDateTime::from(ts_range).reversed_axis();

        let mut chart = ChartBuilder::on(&b)
            .set_label_area_size(LabelAreaPosition::Left, margins.0)
            .set_label_area_size(LabelAreaPosition::Right, margins.0)
            .set_label_area_size(LabelAreaPosition::Top, margins.1)
            .set_label_area_size(LabelAreaPosition::Bottom, margins.1)
            .build_cartesian_2d::<_, _>(rc.freq_low..rc.freq_high, y_axis)?;
        chart
            .configure_mesh()
            .x_desc("Frequency (MHz)")
            .x_label_formatter(&|&x| (x as f32/1e6).round().to_string())
            .draw()?;

        chart.plotting_area().strip_coord_spec()
    };

    let lut = build_lut(settings.colormap);
    let range = settings.power_max - settings.power_min;
    let scale = (lut.len()-1) as f32 / range;

    for record in &rc.records {
        let mut x = (((record.freq_low - rc.freq_low) as f32 / rc.freq_step) + 0.5) as u32;
        let y = rc.timestamps[&NaiveDateTime::new(record.date, record.time)];
        for sample in &record.samples {
            let scaled_pixel = (sample - settings.power_min) * scale;
            let value = &lut[(scaled_pixel as usize).clamp(0, lut.len()-1)];
            plot_area.draw_pixel((x as i32, y as i32), value)?;
            x += 1;
        }
    }

    Ok(())
}
