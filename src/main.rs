use std::error::Error;
use std::process;
use clap::{Arg, ArgMatches, App, crate_version, value_t};

mod csv;
mod draw;

fn heatmap(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let input_path = matches.value_of("INPUT").unwrap();
    let output_path = matches.value_of("OUTPUT").unwrap();
    let power_min = value_t!(matches, "power-min", f32).unwrap_or_else(|e| e.exit());
    let power_max = value_t!(matches, "power-max", f32).unwrap_or_else(|e| e.exit());
    let colormap = matches.value_of("colormap").unwrap();

    let rc = csv::load_records(input_path)?;
    let maps = draw::colormaps();
    let settings = draw::DrawSettings{
        colormap: &maps[colormap],
        power_min: power_min,
        power_max: power_max,
    };
    draw::draw_image(&rc, output_path, &settings)?;
    Ok(())
}

fn main() {
    let matches = App::new("heatmap")
        .about("Plots spectrogram from hackrf_sweep, soapy_power, or rtl_power output.")
        .version(crate_version!())
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
        .arg(Arg::with_name("colormap")
             .long("colormap")
             .possible_values(&draw::colormaps().keys().map(|&x| x).collect::<Vec<_>>())
             .default_value("viridis"))
        .get_matches();

    if let Err(err) = heatmap(&matches) {
        println!("error running heatmap: {}", err);
        process::exit(1);
    }
}
