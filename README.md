# osm-wate.rs
`osm-waters` allows you to get the resulting polygons from the intersection of OSM water bodies and your chosen geometries.

## Install
To install osm-wate.rs:
```bash
cargo install osm-waters

# Or if building from source:
cargo install --path .
```
> Go [here](https://doc.rust-lang.org/cargo/getting-started/installation.html) to install Cargo.

## Usage
Below are some examples on how to use `osm-waters`:

#### Example 1: basic
```bash
osm-waters --target my_target.geojson --water water_polygons.shp --output intersecting_geometries.geojson
```

#### Example 2: let osm-waters download the OSM water data
```bash
osm-waters --target example.geojson --output intersecting_geometries.geojson --download
```

#### Example 3: keep the download files
```bash
osm-waters --target example.geojson --output intersecting_geometries.geojson --download --keep
```

#### Flags
* `--target` takes a GeoJSON or SQL file that queries geometries from a database. See the */tests* directory for examples.
* `--water` takes the OSM water polygons from [OSM water polygons](https://osmdata.openstreetmap.de/data/water-polygons.html). This file **must** be a shapefile.
* `--output` is used to set the path of where the resulting `GeoJSON` file will be saved.
* `--download` downloads the OSM water data
* `--keep` keeps the downloaded data
* `--srid` is used to choose the coordinate system. The OSM water file provided must be in such such srid. In addition, the output will also be in the chosen srid. Default srid is 4326.


# Future improvements
* Reintroduced database option - include choose **geom**.
* Improve error handling.
* Args need better parsing/validation.
* Allow outputs other than GeoJSON, this may be `geoparquet`, `duckdb` or `shapefile`.
* Get `osm-waters` to Homebrew.

## License

See [`LICENSE`](./LICENSE)
