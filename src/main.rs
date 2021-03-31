use std::error::Error;
use std::process;
use clap::{Arg, App, value_t};

mod csv;
mod draw;

fn heatmap(input_path: &str, output_path: &str, power_min: f32, power_max: f32) -> Result<(), Box<dyn Error>> {
    let rc = csv::load_records(input_path)?;
    let settings = draw::DrawSettings{
        power_min: power_min,
        power_max: power_max,
    };
    draw::draw_image(&rc, output_path, &settings)?;
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
