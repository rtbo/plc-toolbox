// Build script for the OPC UA crate
// Reads CSV/XML schema files and generates Rust code accordingly

use std::env;
use std::path;

mod data_types;
mod status_codes;

fn main() {
    println!("cargo:rerun-if-changed=schema/Opc.Ua.Types.bsd");

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = path::Path::new(&out_dir);
    let schema_dir = path::Path::new("schema");

    status_codes::generate_rs(schema_dir, out_dir);
    data_types::generate_rs(schema_dir, out_dir);
}
