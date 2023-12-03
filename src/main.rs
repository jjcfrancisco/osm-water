use geo::{Polygon, LineString, Intersects};
use shapefile as shp;
use shapefile::PolygonRing::{Outer, Inner};
use postgres::{Client, NoTls};
use std::env;
use wkt;
use std::fs;

fn to_geo_poly(polygon: shapefile::Polygon) -> Polygon {

    let mut x: f64;
    let mut y: f64;
    let mut outer_placeholder: Vec<(f64,f64)> = Vec::new();
    let mut inner_placeholder: Vec<LineString> = Vec::new();

    for ring_type in polygon.rings() {
        match ring_type {
            Outer(o) => {
                //Gather all outer rings
                for point in o {
                    x = point.x;
                    y = point.y;
                    outer_placeholder.push((x,y));
                }
            },
            Inner(i) => {
                //Gather all inners
                let mut single_inner_placeholder: Vec<(f64,f64)> = Vec::new();
                for point in i {
                    x = point.x;
                    y = point.y;
                    single_inner_placeholder.push((x,y));
                }
                let ls = LineString::from(single_inner_placeholder);
                inner_placeholder.push(ls);
            },
        }
    }
    
    let ext_ring = LineString::from(outer_placeholder);
    if inner_placeholder.is_empty() {
        Polygon::new(ext_ring, vec![])
    } else {
        Polygon::new(ext_ring, inner_placeholder)
    }

}

#[derive(Debug)]
struct Region {
    project_name: String,
    geom: Polygon,
}

fn postgis_data(query: String) -> Vec<Region> {
    
    let pgcon = env::var("PGCON").expect("$PGCON is not set");
    let mut client = Client::connect(&pgcon, NoTls).unwrap();
    let mut regions: Vec<Region> = Vec::new();
    for row in &client.query(&query, &[]).unwrap() {
        let wkt_geom: String = row.get("geom");
        let result =  wkt::TryFromWkt::try_from_wkt_str(&wkt_geom);
        if result.is_ok() {
            let geom:Polygon = result.unwrap();
            regions.push(Region{
                project_name: row.get("project_name"),
                geom,
            });
        }
    }
    regions
}

fn intersects(polys:Vec<Polygon>, regions:Vec<Region>) -> Vec<Polygon> {

    let mut intersects:Vec<Polygon> = Vec::new();
    for poly in polys.iter() {
        for region in regions.iter() {
            if poly.intersects(&region.geom) {
               intersects.push(poly.to_owned()); 
            }
        }
    }
    intersects

}


fn main() {

    // Too long, use map, if_else
    let query: String;
    let file = fs::read_to_string("src/query.sql");
    if file.is_ok() {
        let reader = file.unwrap();
        let content = reader.parse();
        if content.is_ok() {
            query = content.unwrap();
            // Sort out scope
            let regions = postgis_data(query);
            println!("{:?}", regions)
        }
    }

    //let regions = postgis_data(query);
    //let mut polys:Vec<Polygon> = Vec::new();
    //let filepath = "/Users/frankjimenez/tests/water/shp/water_polygons.shp";
    //let reader = shp::Reader::from_path(filepath);
    //if reader.is_ok() {
    //    let mut content = reader.unwrap();
    //    let shapes = content.iter_shapes_and_records_as::<shp::Polygon, shp::dbase::Record>();
    //    for shape in shapes {
    //        if shape.is_ok() {
    //            // Polygon shape only, record ignored
    //            let (polygon, _) = shape.unwrap();
    //            let poly = to_geo_poly(polygon);
    //            polys.push(poly); 
    //        }
    //    }
    //}

    //let result = intersects(polys, regions);
}
