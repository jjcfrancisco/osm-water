# osm-wate.rs
osm-wate.rs allows you to get the resulting polygons from the intersection of OSM water bodies and your chosen geometries.
In time, this will be a nice looking README page.

## Install
To intall `osm-wate.rs` you must have `cargo` installed. Then simply `cargo install osm-waters`. To install locally use `cargo install --path .`

## Usage
Currently, all flags are compulsory. This may change in future versions. Below is an example of how to use `osm-wate.rs`:

```bash
osm-waters --uri postgresql://user:password@localhost:5432/my_db --sql query.sql --shp og-water-polys.shp --output intersected-water-polys.geojson
```

#### Flags
* The `--uri` requires valid credentials to your database i.e. `postgres://postgres:mypassword@localhost:5432/my_db`.
* The `--sql` query takes a valid sql that used to obtain the geometries from a given database i.e. `SELECT * FROM my_geometries;`
* The `--shp` flag takes the OSM water polygons from [OSM water polygons](https://osmdata.openstreetmap.de/data/water-polygons.html)
* The `--output` flag is used to set the path of where the resulting `GeoJSON` file will be saved.


# Future improvements
* A flag to choose the name of the geometry column from a database - currently this must be `geom` column.
* Improve error handling when wrong file path is introduced.
* Do unit tests.
* Args need parsing/validation.
* Allow outputs other than GeoJSON, this may be `geoparquet`, `duckdb` or `shapefile`.
