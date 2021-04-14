use std::error::Error;

use std::collections::HashMap;

use chrono::naive::NaiveDateTime;
use super::coord::IntoReversedAxis;
use plotters::{chart::ChartBuilder, prelude::{BitMapBackend, IntoDrawingArea, LabelAreaPosition, RangedDateTime}, style::{WHITE}};
pub use colorous::Gradient;

use super::csv::RecordCollection;

pub struct DrawSettings {
    pub colormap: Gradient,
    pub power_min: f32,
    pub power_max: f32,
    pub hide_axes: bool,
}

pub fn colormaps() -> HashMap<&'static str, Gradient> {
    let mut maps: HashMap<&'static str, Gradient> = HashMap::new();
    maps.insert("viridis", colorous::VIRIDIS);
    maps.insert("magma",   colorous::MAGMA);
    maps.insert("inferno", colorous::INFERNO);
    maps.insert("plasma",  colorous::PLASMA);
    maps
}

fn build_lut(colormap: &Gradient) -> Vec<plotters::style::RGBColor> {
    let mut lut = Vec::with_capacity(u16::MAX as usize + 1);
    for i in 0..u16::MAX as usize + 1 {
        let value = colormap.eval_continuous(i as f64 / u16::MAX as f64);
        lut.push(plotters::style::RGBColor{0: value.r, 1: value.g, 2: value.b});
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

    let lut = build_lut(&settings.colormap);
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
