use clap::{ArgAction, Parser};
use io::to_geojson;
use std::env;
use std::path::{Path, PathBuf};
mod database;
mod io;
mod utils;

/// Get polygons from OSM water that intersect with the target geometries and output results in GeoJSON.
#[derive(Parser, Debug)]
#[command(author = "jjcfrancisco", version = "0.1.1", about, long_about = None)]
struct Cli {
    /// Filepath to GeoJSON, Shapefile or SQL
    #[arg(long)]
    target: String,

    /// Filepath to save output file
    #[arg(short, long)]
    output: String,

    ///// Connection string to a database if using SQL as target
    //#[arg(long)]
    //uri: Option<String>,

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
}

fn main() {
    let args = Cli::parse();

    // args need better parsing
    //let uri: Option<String> = args.uri;
    let target: String = args.target;
    let water: Option<String> = args.water;
    let output: String = args.output;
    let srid: Option<String> = args.srid;
    let download: Option<bool> = args.download;

    // Set path to current dir
    let result = env::set_current_dir(Path::new("./"));
    if result.is_err() {
        println!("\nError setting current directory.");
        std::process::exit(1);
    }

    // Workflow
    if utils::check_provided_output(&output) {
        let target_geom = io::open_target(&target);
        let water_data = match download {
            Some(b) => if b {
                io::download_unzip_read(srid)
            } else {
                io::open_shapefile(PathBuf::from(water.expect("Error unpacking 'water' flag.")))
            },
            None => io::open_shapefile(PathBuf::from(water.expect("Error unpacking 'water' flag."))),
        };
        let output_data = match water_data {
            Ok(wd) => utils::geom_intersects(wd, target_geom),
            Err(e) => panic!("Error finding geometry intersects: {}", e)
        };
        match output_data {
            Some(od) => to_geojson(&output, od),
            None => println!("No water bodies intersected with the target geometries")
        };
    }
}
