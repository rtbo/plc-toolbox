use std::env;
use std::path::{Path, PathBuf};

use tokio::{fs, io};

async fn create_new_file(path: &Path) -> io::Result<io::BufWriter<fs::File>> {
    let file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .await?;
    Ok(io::BufWriter::new(file))
}

fn env_dir(var: &str) -> PathBuf {
    env::var(var)
        .map(PathBuf::from)
        .unwrap_or_else(|_| panic!("{} not set", var))
}

async fn status_codes(schema_path: &Path, out_dir: &Path) {
    let csv_path = schema_path.join("StatusCode.csv");

    println!("cargo:rerun-if-changed={}", csv_path.display());

    let status_codes = ua_nsc::parse::status_code::parse_csv(&csv_path)
        .await
        .expect("Failed to parse StatusCode.csv");

    // Generate status_code.rs
    let rs_path = out_dir.join("status_code.rs");
    let mut rs_file = create_new_file(&rs_path).await.unwrap();
    ua_nsc::rs::status_code::generate(&status_codes, &mut rs_file)
        .await
        .unwrap();
}

async fn attribute_ids(schema_path: &Path, out_dir: &Path) {
    let csv_path = schema_path.join("AttributeIds.csv");

    println!("cargo:rerun-if-changed={}", csv_path.display());

    let attr_ids = ua_nsc::parse::attribute_ids::parse_csv(&csv_path)
        .await
        .expect("Failed to parse AttributeIds.csv");

    // Generate status_code.rs
    let rs_path = out_dir.join("attribute_id.rs");
    let mut rs_path = create_new_file(&rs_path).await.unwrap();
    ua_nsc::rs::attribute_id::generate(&attr_ids, &mut rs_path)
        .await
        .unwrap();
}

#[tokio::main]
async fn main() {
    let manifest_dir = env_dir("CARGO_MANIFEST_DIR");
    let out_dir = env_dir("OUT_DIR");
    let schema_path = manifest_dir.parent().unwrap().join("schema");

    status_codes(&schema_path, &out_dir).await;
    attribute_ids(&schema_path, &out_dir).await;
}
