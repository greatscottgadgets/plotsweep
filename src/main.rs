use std::error::Error;
use std::process;
use clap::{Arg, ArgMatches, App, crate_version, value_t};

mod csv;
mod draw;
mod coord;

fn plot(matches: &ArgMatches) -> Result<(), Box<dyn Error>> {
    let input_path = matches.value_of("INPUT").unwrap();
    let output_path = matches.value_of("OUTPUT").unwrap();
    let power_min = value_t!(matches, "power-min", f32).unwrap_or_else(|e| e.exit());
    let power_max = value_t!(matches, "power-max", f32).unwrap_or_else(|e| e.exit());
    let colormap = matches.value_of("colormap").unwrap();

    let rc = csv::load_records(input_path)?;
    let maps = draw::colormaps();
    let settings = draw::DrawSettings{
        colormap: maps[colormap],
        power_min,
        power_max,
        hide_axes: matches.is_present("hide_axes"),
    };
    draw::draw_image(&rc, output_path, &settings)?;
    Ok(())
}

fn main() {
    let matches = App::new("plotsweep")
        .about("A tool to plot spectrogram images using hackrf_sweep, soapy_power, or rtl_power output.")
        .version(crate_version!())
        .arg(Arg::with_name("INPUT")
             .required(true))
        .arg(Arg::with_name("OUTPUT")
             .required(true))
        .arg(Arg::with_name("colormap")
             .long("colormap")
             .possible_values(&draw::colormaps().keys().copied().collect::<Vec<_>>())
             .default_value("viridis"))
        .arg(Arg::with_name("hide_axes")
                  .help("Hide axes")
                  .long("hide-axes")
                  .takes_value(false))
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

    if let Err(err) = plot(&matches) {
        println!("error: {}", err);
        process::exit(1);
    }
}
