# osm-water
In time, this will be a nice looking README page.

# Usage
```bash
cargo run -- --uri postgresql://user:password@localhost:5432/my_db --sql query.sql --shp og-water-polys.shp --output intersected-water-polys.geojson
```

# Improvements
* Needs parsing of args
* Allow input sources other than PostGIS
* Allow outputs other than GeoJSON