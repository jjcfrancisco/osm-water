use geo_types::GeometryCollection;
use postgres::{Client, NoTls};

// Reads the geometries from a database
pub fn postgis_data(pgcon: Option<String>, query: String) -> GeometryCollection {
    if pgcon.is_some() {
        let mut client = Client::connect(&pgcon.unwrap(), NoTls).unwrap();
        let mut features: Vec<geo::Geometry> = Vec::new();
        for row in &client.query(&query, &[]).unwrap() {
            let wkt_geom: String = row.get("geom");
            let result = wkt::TryFromWkt::try_from_wkt_str(&wkt_geom);
            if result.is_ok() {
                let geom: geo::Geometry = result.unwrap();
                features.push(geom);
            }
        }

        let gc = GeometryCollection::new_from(features);
        return gc;
    } else {
        eprintln!("\nError when reading sql file.");
        std::process::exit(1)
    }
}
