# osm-wate.rs
osm-wate.rs allows you to get the resulting polygons from the intersection of OSM water bodies and your chosen geometries.

## Install
To install osm-wate.rs:
```bash
cargo install osm-waters

# Or if building from source:
cargo install --path .
```
> Cargo must be installed.

## Usage
Below are some examples on how to use osm-wate.rs:

#### GeoJSON example
```bash
--target ./example.geojson --input ~/water_polygons.shp --output intersecting_geometries.geojson
```

#### Database example
```bash
osm-waters --target ./query.sql --uri postgresql://postgres:mypassword@localhost:5432/mydatabase --input ./water_polygons.shp --output intersecting_geometries.geojson
```

#### Flags
* The `--target` takes a GeoJSON or SQL file that queries geometries from a database. See the */tests* directory for examples.
* The `--uri` requires credentials to your database. This is an optional flag but compulsory if using a database as target.
* The `--input` takes the OSM water polygons from [OSM water polygons](https://osmdata.openstreetmap.de/data/water-polygons.html). Currently, this file **must** be a shapefile.
* The `--output` is used to set the path of where the resulting `GeoJSON` file will be saved.


# Future improvements
* A flag to choose the name of the geometry column from a database - currently this must be `geom` column.
* Improve error handling when a wrong file path is passed.
* Do unit tests.
* Args need better parsing/validation.
* Allow the use of a database as input.
* Allow outputs other than GeoJSON, this may be `geoparquet`, `duckdb` or `shapefile`.
