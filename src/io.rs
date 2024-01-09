use crate::database::postgis_data;
use crate::utils::to_geo;
use geo_types::GeometryCollection;
use geojson::{quick_collection, GeoJson};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

// Reads shapefile
pub fn read_shapefile(filepath: &str) -> HashMap<String, geo::Polygon> {
    let mut polys: HashMap<String, geo::Polygon> = HashMap::new();
    let reader = shapefile::Reader::from_path(filepath);
    if reader.is_ok() {
        let mut content = reader.unwrap();
        let shapes =
            content.iter_shapes_and_records_as::<shapefile::Polygon, shapefile::dbase::Record>();
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
pub fn to_geojson(output_path: &str, targets: Vec<geo::Polygon>) {
    let mut features: Vec<geojson::Feature> = Vec::new();

    for target in targets.iter() {
        let geometry = geojson::Geometry::new(geojson::Value::from(target));
        let mut properties = geojson::JsonObject::new();
        properties.insert(String::from("name"), geojson::JsonValue::Null);

        let feature = geojson::Feature {
            bbox: None,
            geometry: Some(geometry),
            id: None,
            properties: Some(properties),
            foreign_members: None,
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
        return quick_collection(&d).unwrap();
    } else {
        eprintln!("\nError when reading geojson file.");
        std::process::exit(1);
    }
}

pub fn open(filepath: &str, uri: Option<String>) -> GeometryCollection {
    // Allowed file extensions
    let allowed = vec!["geojson", "sql"];

    // Finds file extension provided by user
    let file_ext = Path::new(filepath).extension().and_then(OsStr::to_str);

    // Opens file depending on file type
    if file_ext.is_some() {
        let is_allowed = allowed
            .iter()
            .any(|&x| file_ext.unwrap().to_lowercase() == x);

        if is_allowed && file_ext.unwrap() == "geojson" {
            open_geojson(filepath)
        } else if is_allowed && file_ext.unwrap() == "sql" {
            if uri.is_none() {
                eprintln!("\nA valid uri must be provided.");
                std::process::exit(1)
            };
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
