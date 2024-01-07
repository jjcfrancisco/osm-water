use clap::Parser;
use geo::{*};
use std::collections::HashMap;
use std::{env, fs};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::ffi::OsStr;
use geojson::{GeoJson, quick_collection};
use geo_types::GeometryCollection;
use postgres::{Client, NoTls};

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

// Geometries are transformed to GeoRust: Geo
fn to_geo(polygon: shapefile::Polygon) -> geo::Polygon {

    let mut outer_placeholder: Vec<(f64,f64)> = Vec::new();
    let mut inner_rings: Vec<geo::LineString> = Vec::new();

    for ring_type in polygon.rings() {
        match ring_type {
            //Gather all outer rings
            shapefile::PolygonRing::Outer(out) => out.iter().for_each(|p| {outer_placeholder.push((p.x,p.y))}),
            //Gather all inner rings
            shapefile::PolygonRing::Inner(inn) => {
                let mut inner_ring: Vec<(f64,f64)> = Vec::new();
                inn.iter().for_each(|p| {inner_ring.push((p.x,p.y))});
                let ls = geo::LineString::from(inner_ring);
                inner_rings.push(ls);
            },
        }
    }
    
    let outer_ring = geo::LineString::from(outer_placeholder);
    if inner_rings.is_empty() {
        geo::Polygon::new(outer_ring, vec![])
    } else {
        geo::Polygon::new(outer_ring, inner_rings)
    }

}

struct Feature {
    geom: geo::Polygon,
}

// Reads the geometries from a database
fn postgis_data(pgcon: Option<String>, query: String) -> GeometryCollection {

    if pgcon.is_some() {

        let mut client = Client::connect(&pgcon.unwrap(), NoTls).unwrap();
        let mut features: Vec<Feature> = Vec::new();
        for row in &client.query(&query, &[]).unwrap() {
            let wkt_geom: String = row.get("geom");
            let result =  wkt::TryFromWkt::try_from_wkt_str(&wkt_geom);
            if result.is_ok() {
                let geom: geo::Polygon = result.unwrap();
                features.push(Feature{
                    geom,
                });
            }
        }

        //features
        //Dummy text below
        let polygon = geo::Polygon::new(LineString::from(vec![(0., 0.), (1., 1.), (1., 0.), (0., 0.)]), vec![]);
        let gc = GeometryCollection::from_iter(vec![polygon.to_owned(), polygon.to_owned()]);
        return gc

    } else {
        eprintln!("\nError when reading sql file.");
        std::process::exit(1)
    }

}

// Iterates over interesects
fn geom_intersects(water_polys:HashMap<String, geo::Polygon>, target_polys:GeometryCollection) -> Vec<geo::Polygon> {

    let mut result:Vec<geo::Polygon> = Vec::new();

    for (_, water_poly) in &water_polys {
        for target_poly in &target_polys {

            if let Ok(p) = geo::Polygon::try_from(target_poly.to_owned()) {
                if water_poly.intersects(&p) {
                    result.push(water_poly.to_owned());
                }
            } else if let Ok(mp) = geo::MultiPolygon::try_from(target_poly.to_owned()) {
                for p in mp {
                    if water_poly.intersects(&p) {
                        result.push(water_poly.to_owned());
                    }
                }
            }
        }
    }

    // Removes duplicates
    result.dedup();
    result

}

// Reads shapefile
fn read_shapefile(filepath: &str) -> HashMap<String, geo::Polygon> {

    let mut polys:HashMap<String, geo::Polygon> = HashMap::new();
    let reader = shapefile::Reader::from_path(filepath);
    if reader.is_ok() {
        let mut content = reader.unwrap();
        let shapes = content.iter_shapes_and_records_as::<shapefile::Polygon, shapefile::dbase::Record>();
        for (ind, shape) in shapes.enumerate() {
            if shape.is_ok() {
                // Polygon shape only, record ignored
                let (polygon, _) = shape.unwrap();
                let poly = to_geo(polygon);
                polys.insert(ind.to_string(), poly);
            }
        }
    }
    polys

}

// To GeoJSON object
fn to_geojson(output_path: &str, targets: Vec<geo::Polygon>) {

    let mut features:Vec<geojson::Feature> = Vec::new();

    for target in targets.iter() {
        let geometry = geojson::Geometry::new(geojson::Value::from(target));
        let mut properties = geojson::JsonObject::new();
        properties.insert(
            String::from("name"),
            geojson::JsonValue::Null,
        );

        let feature = geojson::Feature {
            bbox: None,
            geometry: Some(geometry),
            id: None,
            properties: Some(properties),
            foreign_members: None
        };

        features.push(feature)

    }

    let feature_collection = geojson::FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    };

    let geojson = geojson::GeoJson::from(feature_collection);
    let geojson_string = geojson.to_string();
    let result = fs::write(output_path, geojson_string);
    match result {
        Ok(_) => println!("\nGeoJSON succesfully saved.\n"),
        Err(e) => println!("{:?}", e),
    }

}

fn open_sql(filepath: &str, uri: Option<String>) -> GeometryCollection<f64> {

    let query = fs::read_to_string(filepath);


    if query.is_ok() {
        postgis_data(uri, query.unwrap())
    } else {
        eprintln!("\nError when reading sql file.");
        std::process::exit(1)
    }

}

fn open_geojson(filepath: &str) -> GeometryCollection<f64> {

    let mut file = File::open(&filepath).expect("Wrong file path provided.");
    let mut file_contents = String::new();
    let _ = file.read_to_string(&mut file_contents);

    let data = file_contents.parse::<GeoJson>();
    
    if let Ok(d) = data {
        return quick_collection(&d).unwrap()
    } else {
        eprintln!("\nError when reading geojson file.");
        std::process::exit(1);
    }

}

fn check_provided_output(filepath: &str) -> bool {

    // Allowed file extensions
    let allowed = vec!["geojson"];

    // Finds file extension provided by user
    let file_ext = Path::new(filepath)
        .extension()
        .and_then(OsStr::to_str);

    if file_ext.is_some() {
        let is_allowed = allowed.iter()
                                .any(|&x| file_ext.unwrap()
                                                  .to_lowercase() == x);

        if is_allowed && file_ext.unwrap() == "geojson" {
            return true
        } else {
            eprintln!("\nProvided output file type not allowed. It must be geojson.");
            std::process::exit(1)
        }
    } else {
        eprintln!("\nError when using the provided file path.");
        std::process::exit(1)
    }

}

fn open(filepath: &str, uri: Option<String>) -> GeometryCollection {

    // Allowed file extensions
    let allowed = vec!["geojson", "sql"];

    // Finds file extension provided by user
    let file_ext = Path::new(filepath)
        .extension()
        .and_then(OsStr::to_str);

    // Opens file depending on file type
    if file_ext.is_some() {
        let is_allowed = allowed.iter()
                                .any(|&x| file_ext.unwrap()
                                                  .to_lowercase() == x);

        if is_allowed && file_ext.unwrap() == "geojson" {
            open_geojson(filepath)
        } else if is_allowed && file_ext.unwrap() == "sql" {
            open_sql(filepath, uri)
        } else {
            eprintln!("\nFile type provided not allowed.");
            std::process::exit(1)
        }
        
    } else {
        eprintln!("\nError when using the provided file path.");
        std::process::exit(1)
    }

}

fn main() {

    let args = Cli::parse();
    let target:String = args.target; // parse
    let input:String = args.input; // parse
    let output:String = args.output; // parse
    let uri:Option<String> = args.uri; // parse

    // Set path to current dir
    let result = env::set_current_dir(Path::new("./"));
    if result.is_err() {
        println!("\nError setting current directory.");
        std::process::exit(1);
    }

    if check_provided_output(&output) {
        let target_geom = open(&target, uri);
        let osm_water_polys = read_shapefile(&input);
        let result = geom_intersects(osm_water_polys, target_geom);
        to_geojson(&output, result)
    }

}
