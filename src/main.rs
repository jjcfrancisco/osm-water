use clap::Parser;
use std::path::Path;
use std::env;
mod database;
mod io;
mod utils;


/// Get polygons from OSM water that intersect with the target geometries and output results in GeoJSON.
#[derive(Parser, Debug)]
#[command(author = "jjcfrancisco", version = "0.1.1", about, long_about = None)]
struct Cli {
    /// Connection string to a database if using SQL as target
    #[arg(long)]
    uri: Option<String>,

    /// Filepath to GeoJSON, Shapefile or SQL
    #[arg(long)]
    target: String,

    /// Filepath to OSM water shapefile
    #[arg(long)]
    input: String,

    /// Filepath to save output file
    #[arg(short, long)]
    output: String,
}

fn main() {
    let args = Cli::parse();

    // args need better parsing
    let target: String = args.target;
    let input: String = args.input;
    let output: String = args.output;
    let uri: Option<String> = args.uri;

    // Set path to current dir
    let result = env::set_current_dir(Path::new("./"));
    if result.is_err() {
        println!("\nError setting current directory.");
        std::process::exit(1);
    }

    // Workflow
    if utils::check_provided_output(&output) {
        let target_geom = io::open(&target, uri);
        let osm_water_polys = io::read_shapefile(&input);
        let result = utils::geom_intersects(osm_water_polys, target_geom);
        io::to_geojson(&output, result)
    }
}
