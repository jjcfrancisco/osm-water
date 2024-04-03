use geo::Intersects;
use geo_types::GeometryCollection;

// Iterates over interesects
pub fn geom_intersects(
    water_geoms: GeometryCollection,
    target_geoms: GeometryCollection,
) -> Option<GeometryCollection> {
    let mut result: Vec<geo::Geometry> = Vec::new();

    println!("\n Finding intersecting geometries...\n");

    for water_geom in &water_geoms {
        for target_geom in &target_geoms {
            if let Ok(p) = geo::Polygon::try_from(target_geom.to_owned()) {
                if water_geom.intersects(&p) {
                    result.push(water_geom.to_owned());
                }
            } else if let Ok(mp) = geo::MultiPolygon::try_from(target_geom.to_owned()) {
                for p in mp {
                    if water_geom.intersects(&p) {
                        result.push(water_geom.to_owned());
                    }
                }
            }
        }
    }

    // Removes duplicates
    result.dedup();
    Some(geo::GeometryCollection::new_from(result))
}

// Geometries are transformed to GeoRust: Geo
pub fn to_geo(polygon: shapefile::Polygon) -> geo::Geometry {
    let mut outer_placeholder: Vec<(f64, f64)> = Vec::new();
    let mut inner_rings: Vec<geo::LineString> = Vec::new();

    for ring_type in polygon.rings() {
        match ring_type {
            //Gather all outer rings
            shapefile::PolygonRing::Outer(out) => {
                out.iter().for_each(|p| outer_placeholder.push((p.x, p.y)))
            }
            //Gather all inner rings
            shapefile::PolygonRing::Inner(inn) => {
                let mut inner_ring: Vec<(f64, f64)> = Vec::new();
                inn.iter().for_each(|p| inner_ring.push((p.x, p.y)));
                let ls = geo::LineString::from(inner_ring);
                inner_rings.push(ls);
            }
        }
    }

    let outer_ring = geo::LineString::from(outer_placeholder);
    if inner_rings.is_empty() {
        let poly = geo::Polygon::new(outer_ring, vec![]);
        geo::Geometry::from(poly)
    } else {
        let poly = geo::Polygon::new(outer_ring, inner_rings);
        geo::Geometry::from(poly)
    }
}

