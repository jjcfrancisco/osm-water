use clap::Parser;
use geo::{*};
use shapefile;
use postgres::{Client, NoTls};
use wkt;
use std::{env, fs};
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::ffi::OsStr;
use geojson::GeoJson;

/// Get polygons from OSM water that intersect with the target geometries and output results in GeoJSON.
#[derive(Parser, Debug)]
#[command(author = "jjcfrancisco", version = "0.1.1", about, long_about = None)]
struct Cli {

    ///// Connection string to a database if using SQL as target
    //#[arg(long)]
    //uri: Option<String>,

    /// Filepath to GeoJSON, Shapefile or SQL
    #[arg(long)]
    target: String,

    ///// Filepath to OSM water shapefile
    //#[arg(long)]
    //input: String,

    ///// Filepath to save output file
    //#[arg(short, long)]
    //output: String,

}

// Geometries are transformed to GeoRust: Geo
fn to_geo(polygon: shapefile::Polygon) -> geo::Polygon {

    let mut x: f64;
    let mut y: f64;
    let mut outer_placeholder: Vec<(f64,f64)> = Vec::new();
    let mut inner_placeholder: Vec<geo::LineString> = Vec::new();

    for ring_type in polygon.rings() {
        match ring_type {
            shapefile::PolygonRing::Outer(o) => {
                //Gather all outer rings
                for point in o {
                    x = point.x;
                    y = point.y;
                    outer_placeholder.push((x,y));
                }
            },
            shapefile::PolygonRing::Inner(i) => {
                //Gather all inner rings
                let mut single_inner_placeholder: Vec<(f64,f64)> = Vec::new();
                for point in i {
                    x = point.x;
                    y = point.y;
                    single_inner_placeholder.push((x,y));
                }
                let ls = geo::LineString::from(single_inner_placeholder);
                inner_placeholder.push(ls);
            },
        }
    }
    
    let ext_ring = geo::LineString::from(outer_placeholder);
    if inner_placeholder.is_empty() {
        geo::Polygon::new(ext_ring, vec![])
    } else {
        geo::Polygon::new(ext_ring, inner_placeholder)
    }

}

struct Feature {
    geom: geo::Polygon,
}

// Reads the geometries from a database
fn postgis_data(pgcon: &str, query: String) -> Vec<Feature> {
    
    let mut client = Client::connect(&pgcon, NoTls).unwrap();
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

    features

}

// Iterates over interesects
fn intersects(polys:Vec<geo::Polygon>, targets:Vec<Feature>) -> Vec<geo::Polygon> {

    let mut intersects:Vec<geo::Polygon> = Vec::new();
    for poly in polys.iter() {
        for target in targets.iter() {
            if poly.intersects(&target.geom) {
               intersects.push(poly.to_owned()); 
            }
        }
    }

    // Removes duplicates
    intersects.dedup();

    intersects

}

// Reads file
fn read_file(filepath: &str) -> String {

    let path = Path::new(filepath);
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open: {}", why),
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read: {}", why),
        Ok(_) => s,
    }
}

// Reads shapefile
fn read_shapefile(filepath: &str) -> Vec<geo::Polygon> {

    let mut polys:Vec<geo::Polygon> = Vec::new();
    let reader = shapefile::Reader::from_path(filepath);
    if reader.is_ok() {
        let mut content = reader.unwrap();
        let shapes = content.iter_shapes_and_records_as::<shapefile::Polygon, shapefile::dbase::Record>();
        for shape in shapes {
            if shape.is_ok() {
                // Polygon shape only, record ignored
                let (polygon, _) = shape.unwrap();
                let poly = to_geo(polygon);
                polys.push(poly); 
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
        Ok(_) => println!("\n GeoJSON succesfully saved.\n"),
        Err(e) => println!("{:?}", e),
    }

}

fn open_sql(filepath: &str) -> String {

    let data = fs::read_to_string(filepath);
    if data.is_ok() {
        return data.unwrap()
    } else {
        eprintln!("Error when reading sql file.");
        std::process::exit(1)
    }

}

fn open_geojson(filepath: &str) -> GeoJson {

    let mut file = File::open(&filepath).unwrap();
    let mut file_contents = String::new();
    let _ = file.read_to_string(&mut file_contents);

    let data = file_contents.parse::<GeoJson>();

    if data.is_ok() {
        return data.unwrap()
    } else {
        eprintln!("Error when reading geojson file.");
            std::process::exit(1)
    }

}

enum File_type<GeoJson, String> {
    GeoJSON(GeoJson),
    Sql(String),
}

fn open(filepath: &str) -> File_type<GeoJson, String> {

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
            File_type::GeoJSON(open_geojson(filepath))
        } else if is_allowed && file_ext.unwrap() == "sql" {
            File_type::Sql(open_sql(filepath))
        } else {
            eprintln!("File type provided not allowed.");
            std::process::exit(1)
        }
        
    } else {
        eprintln!("Error when using the provided file path.");
        std::process::exit(1)
    }

}

fn main() {

    let args = Cli::parse();
    let target:String = args.target; // parse
    
    //let uri:Option<String> = args.uri; // parse
    //let input:String = args.input; // parse
    //let output:String = args.output; // parse

    // Set path to current dir
    let result = env::set_current_dir(Path::new("./"));
    if result.is_err() {
        println!("Error setting current directory.");
        std::process::exit(1);
    }

    let content = open(&target);
    match content {
        File_type::GeoJSON(g) => println!("{:?}", g),
        File_type::Sql(s) => println!("{:?}", s),
    }


    //let query = read_file(&sql_path);
    //let regions = postgis_data(&uri, query);
    //let polygons = read_shapefile(&water_path);
    //let targets = intersects(polygons, regions);
    //to_geojson(&output_path, targets)

}
