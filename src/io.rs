use crate::geo::to_geo;
use anyhow::Result;
use core::panic;
use geo_types::GeometryCollection;
use geojson::{quick_collection, GeoJson};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::fs::File;
use std::io::copy;
use walkdir::WalkDir;
use zip_extensions::*;
use std::io::prelude::*;

// Cleans up files
pub fn cleanup(srid: &str) -> Result<()> {
    let current_path = std::env::current_dir()?;
    let dir = current_path.join(format!("water-polygons-split-{}", srid));
    let _ = std::fs::remove_dir_all(dir);
    let file = current_path.join(format!("water-polygons-split-{}.zip", srid));
    let _ = std::fs::remove_file(file);
    Ok(())
}

// Deals with download, unzip an read file
pub fn download_unzip_read(srid: &str) -> Result<GeometryCollection> {
    let mut download_path = download_file(srid)?;
    unzip_file(&download_path)?;
    download_path.set_extension("");
    let shp_path = find_file(download_path, "shp")?;
    let shp_data = open_shapefile(shp_path)?;
    Ok(shp_data)
}

fn find_file(dir: PathBuf, ext: &str) -> Result<PathBuf> {
    let mut found: PathBuf = Default::default();

    for entry in WalkDir::new(dir) {
        match entry {
            Ok(f) => {
                let entry_ext = f.path().extension();
                if entry_ext.is_some() {
                    if entry_ext.unwrap() == ext {
                        found = PathBuf::from(f.path())
                    }
                }
            }
            Err(e) => panic!("Could not find shapefiles: {}", e),
        }
    }
    Ok(found)
}

// Unzip OSM water
fn unzip_file(filepath: &PathBuf) -> Result<PathBuf> {
    let target_dir: PathBuf = PathBuf::from(
        filepath
            .parent()
            .expect("Internal Error: could not obtain parent path."),
    );

    let result = zip_extract(&filepath, &target_dir);
    match result {
        Ok(_) => Ok(target_dir),
        Err(e) => panic!("Error: {}", e),
    }
}

// Downloads OSM water
fn download_file(mut srid: &str) -> Result<PathBuf> {
    let url = if srid == "3857" {
        srid = "3857";
        "https://osmdata.openstreetmap.de/download/water-polygons-split-3857.zip"
    } else {
        "https://osmdata.openstreetmap.de/download/water-polygons-split-4326.zip"
    };

    // Send an HTTP GET request to the URL
    let mut response = reqwest::blocking::get(url)?;

    // Create a new file to write the downloaded image to
    let filename = format!("water-polygons-split-{}.zip", srid);
    let current_path = std::env::current_dir()?;
    let fullpath = current_path.join(filename);
    let mut dest = File::create(&fullpath)?;

    // Copy the contents of the response to the file
    copy(&mut response, &mut dest)?;

    Ok(fullpath)
}

// Reads shapefile
pub fn open_shapefile(filepath: PathBuf) -> Result<GeometryCollection> {
    let mut polys: Vec<geo::Geometry> = Vec::new();
    let reader = shapefile::Reader::from_path(filepath);
    if reader.is_ok() {
        let mut content = reader.unwrap();
        let shapes =
            content.iter_shapes_and_records_as::<shapefile::Polygon, shapefile::dbase::Record>();
        for (_, shape) in shapes.enumerate() {
            if shape.is_ok() {
                // Polygon shape only, record ignored
                let (polygon, _) = shape.unwrap();
                let poly = to_geo(polygon);
                polys.push(poly);
            }
        }
        Ok(GeometryCollection::new_from(polys))
    } else {
        panic!("\nError when reading shapefile.")
    }
}

// To GeoJSON object
pub fn to_geojson(output_path: &str, targets: GeometryCollection) {
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
    let result = std::fs::write(output_path, geojson_string);
    match result {
        Ok(_) => println!("\nGeoJSON succesfully saved.\n"),
        Err(e) => println!("{:?}", e),
    }
}

fn open_geojson(filepath: &str) -> GeometryCollection<f64> {
    let mut file = File::open(&filepath).expect("Wrong file path provided.");
    let mut file_contents = String::new();
    let _ = file.read_to_string(&mut file_contents);

    let data = file_contents.parse::<GeoJson>();

    if let Ok(d) = data {
        quick_collection(&d).unwrap()
    } else {
        eprintln!("\nError when reading geojson file.");
        std::process::exit(1);
    }
}

// Opens target geometries
pub fn open_target(filepath: &str) -> GeometryCollection {
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
        } else {
            eprintln!("\nFile type provided not allowed.");
            std::process::exit(1)
        }
    } else {
        eprintln!("\nError when using the provided file path.");
        std::process::exit(1)
    }
}

pub fn open_input(filepath: &str) -> Result<GeometryCollection> {
    // Allowed file extensions
    let allowed = vec!["shp", "sql"];

    // Finds file extension provided by user
    let file_ext = Path::new(filepath).extension().and_then(OsStr::to_str);

    // Opens file depending on file type
    if file_ext.is_some() {
        let is_allowed = allowed
            .iter()
            .any(|&x| file_ext.unwrap().to_lowercase() == x);

        if is_allowed && file_ext.unwrap() == "shp" {
            open_shapefile(PathBuf::from(filepath.to_owned()))
        //} else if is_allowed && file_ext.unwrap() == "sql" {
        //    if uri.is_none() {
        //        eprintln!("\nA valid uri must be provided.");
        //        std::process::exit(1)
        //    };
        //    open_sql(filepath, uri)
        } else {
            eprintln!("\nFile type provided not allowed.");
            std::process::exit(1)
        }
    } else {
        eprintln!("\nError when using the provided file path.");
        std::process::exit(1)
    }
}
