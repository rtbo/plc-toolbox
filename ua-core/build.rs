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

#[tokio::main]
async fn main() {
    let manifest_dir = env_dir("CARGO_MANIFEST_DIR");
    let out_dir = env_dir("OUT_DIR");

    let schema_path = manifest_dir.parent().unwrap().join("schema");
    let status_code_path = schema_path.join("StatusCode.csv");

    println!("cargo:rerun-if-changed={}", status_code_path.display());

    let status_code = ua_nsc::parse::status_code::parse_csv(&status_code_path)
        .await
        .expect("Failed to parse StatusCode.csv");

    // Generate status_code.rs
    let status_code_path = out_dir.join("status_code.rs");
    println!("Writing {}", status_code_path.display());
    let mut status_code_file = create_new_file(&status_code_path).await.unwrap();
    ua_nsc::rs::status_code::generate(&status_code, &mut status_code_file)
        .await
        .unwrap();
}
