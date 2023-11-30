use geo::{Polygon, LineString};
use shapefile as shp;
use shapefile::PolygonRing::{Outer, Inner};

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

fn intersects() {
}

fn main() {

    let filepath = "/Users/frankjimenez/tests/water/shp/water_polygons.shp";
    let reader = shp::Reader::from_path(filepath);
    if shp::Reader::from_path(filepath).is_ok() {
        let mut content = reader.unwrap();
        let shapes = content.iter_shapes_and_records_as::<shp::Polygon, shp::dbase::Record>();
        for shape in shapes {
            if shape.is_ok() {

                // Polygon shape only, record ignored
                let (polygon, _) = shape.unwrap();
                let geo_poly = to_geo_poly(polygon);
                println!("{:?}", geo_poly)
                
            }
        }
    }
}
