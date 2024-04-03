use crate::Path;
use std::ffi::OsStr;

pub fn check_provided_output(filepath: &str) -> bool {
    // Allowed file extensions
    let allowed = vec!["geojson"];

    // Finds file extension provided by user
    let file_ext = Path::new(filepath).extension().and_then(OsStr::to_str);

    if file_ext.is_some() {
        let is_allowed = allowed
            .iter()
            .any(|&x| file_ext.unwrap().to_lowercase() == x);

        if is_allowed && file_ext.unwrap() == "geojson" {
            return true;
        } else {
            eprintln!("\nProvided output file type not allowed. It must be geojson.");
            std::process::exit(1)
        }
    } else {
        eprintln!("\nError when using the provided file path.");
        std::process::exit(1)
    }
}

