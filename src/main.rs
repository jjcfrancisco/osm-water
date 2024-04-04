use clap::{ArgAction, Parser};
use io::to_geojson;
use std::env;
use std::path::{Path, PathBuf};
mod geo;
mod io;
mod validate;
use crate::io::cleanup;
use ctrlc;

/// Get polygons from OSM water that intersect with the target geometries and output results in GeoJSON.
#[derive(Parser, Debug)]
#[command(author = "jjcfrancisco", version = "0.2.1", about, long_about = None)]
struct Cli {
    /// Filepath to GeoJSON, Shapefile or SQL
    #[arg(long)]
    target: String,

    /// Filepath to save output file
    #[arg(short, long)]
    output: String,

    /// Filepath to OSM water shapefile
    #[arg(long)]
    water: Option<String>,

    /// Coordinate system
    #[arg(long, default_value = "4326")]
    srid: Option<String>,

    /// Download water
    #[arg(
        long,
        action(ArgAction::SetTrue),
        default_value = "false",
        required_unless_present = "water"
    )]
    download: Option<bool>,

    /// Keep download
    #[arg(
        long,
        action(ArgAction::SetTrue),
        default_value = "false",
        required_unless_present = "download"
    )]
    keep: Option<bool>,
}

fn main() {
    let args = Cli::parse();

    // args need better parsing
    let target: String = args.target;
    let water: Option<String> = args.water;
    let output: String = args.output;
    let srid: Option<String> = args.srid;
    let download: Option<bool> = args.download;
    let keep: Option<bool> = args.keep;

    let srid_unwrapped = srid.unwrap();
    let ctrlc_handler = srid_unwrapped.clone();

    // Handling Ctrl+C
    ctrlc::set_handler(move || {
        println!("Process cancelled.");
        let _ = cleanup(&ctrlc_handler);
        std::process::exit(1)
    })
    .expect("Error setting Ctrl-C handler");

    // Set path to current dir
    let result = env::set_current_dir(Path::new("./"));
    if result.is_err() {
        panic!("\nError setting current directory.");
    }

    // Workflow
    if validate::check_provided_output(&output) {
        let target_geom = io::open_target(&target);
        let water_data = match download {
            Some(d) => {
                if d {
                    io::download_unzip_read(&srid_unwrapped)
                } else {
                    io::open_shapefile(PathBuf::from(water.expect("Error unpacking 'water' flag.")))
                }
            }
            None => {
                io::open_shapefile(PathBuf::from(water.expect("Error unpacking 'water' flag.")))
            }
        };
        let output_data = match water_data {
            Ok(wd) => geo::geom_intersects(wd, target_geom),
            Err(e) => panic!("Error finding geometry intersects: {}", e),
        };
        match output_data {
            Some(od) => to_geojson(&output, od),
            None => println!("No water bodies intersected with the target geometries"),
        };
        if download.unwrap() & keep.unwrap() == false {
            io::cleanup(&srid_unwrapped).unwrap()
        };
    }
}
