use geo::{Polygon, LineString};
use shapefile::PolygonRing::{Outer, Inner};

fn to_geo_poly(polygon: shapefile::Polygon) -> Polygon {

    let mut placeholder: Vec<(f64, f64)> = Vec::new();
    for points in polygon.into_inner() {
        for point in points.into_inner() {
            let x = point.x;
            let y = point.y;
            placeholder.push((x,y))
        }
    }
    let ext_ring = LineString::from(placeholder);
    Polygon::new(ext_ring, vec![])

}

fn main() {

    let filepath = "/Users/frankjimenez/tests/water/shp/water_polygons.shp";
    let reader = shapefile::Reader::from_path(filepath);
    if reader.is_ok() {
        let mut content = reader.unwrap();
        let shapes = content.iter_shapes_and_records_as::<shapefile::Polygon, shapefile::dbase::Record>();
        for shape in shapes {
            if shape.is_ok() {
                let (polygon, _) = shape.unwrap();
                for ring_type in polygon.rings() {
                    match ring_type {
                        Outer(o) => {
                            //Gather all outers
                        },
                        Inner(i) => {
                            //Gather all inners
                        },
                    }
                }

                //for points in polygon.into_inner() {
                //    let mut placeholder: Vec<(f64, f64)> = Vec::new();
                //    for point in points.into_inner() {
                //        let x = point.x;
                //        let y = point.y;
                //        placeholder.push((x,y))
                //    }
                //    let ext_ring = LineString::from(placeholder);
                //    let poly = Polygon::new(ext_ring, vec![]);
                //    //println!("{:?}", poly)
                //}
                //std::process::exit(0);
            }
        }
    }






    // Good one
    //let path = "/Users/frankjimenez/tests/water/shp/water_polygons.shp";
    //let mut reader = shapefile::Reader::from_path(path).unwrap();
    //for result in reader.iter_shapes_and_records_as::<shapefile::Polygon, shapefile::dbase::Record>() {
    //        let (shape, _record) = result.unwrap();
    //        for vpoints in shape.into_inner() {
    //            let mut Ss = String::new();
    //            for ponto in vpoints.into_inner() {
    //                let s1: String = ponto.x.to_string();
    //                let s2: String = ponto.y.to_string();
    //                Ss.push_str(&s1);
    //                Ss.push_str(",");
    //                Ss.push_str(&s2);
    //            }
    //            println!("{}", Ss);
    //            std::process::exit(0);
    //        }
    //    }










    //let reader = shapefile::ShapeReader::from_path(path);
    //if reader.is_ok() {
    //    let content = reader.unwrap();
    //    match content.read_as::<shapefile::Polygon>() {
    //        Ok(shapes) => {
    //            let mut points = shapes.first().into_iter();
    //            for point in points {
    //                println!("{}",point) 
    //            }
    //        },
    //        Err(e) => (),
    //    }
    //} else {
    //    println!("Error when unwrapping shapefile")
    //}


    //for shape in reader.iter_shapes() {
    //    match shape? {
    //        shapefile::Shape::Multipatch(shp) => println!("Multipoint!"),
    //        _ => println!("Other type of shape"),
    //    }
    //}


    //let path = "/Users/frankjimenez/tests/water/shp/water_polygons.shp";
    //let mut reader = Reader::from_path(path).unwrap();
    //let polygons = reader.read_as::<shapefile::Polygon, shapefile::dbase::Record>().unwrap();
    //for (shape, record) in polygons {
    //    println!("{}", shape);
    //    for (name, value) in record {
    //        println!("{}, {}", name, value)
    //    }
    //}

}
