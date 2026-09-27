use std::io::{self, BufRead, Write};
use std::path;

struct NodeId {
    name: String,
    id: u32,
}

fn parse_node_ids(csv_path: &path::Path) -> Vec<NodeId> {
    let mut node_ids = Vec::new();

    let file = std::fs::File::open(&csv_path).unwrap();
    let reader = io::BufReader::new(file);
    for line in reader.lines() {
        let line = line.unwrap();
        let mut parts = line.split(',');
        let name = parts.next().unwrap().to_string();
        let id = parts.next().unwrap().parse::<u32>().unwrap();
        node_ids.push(NodeId { name, id });
    }
    node_ids
}

pub fn generate_rs(schema_dir: &path::Path, out_dir: &path::Path) {
    let csv_path = schema_dir.join("NodeIds.csv");
    println!("cargo:rerun-if-changed={}", csv_path.display());

    let node_ids = parse_node_ids(&csv_path);

    let out_path = out_dir.join("ns0.rs");
    let out_file = std::fs::File::create(&out_path).unwrap();
    let mut output = std::io::BufWriter::new(out_file);

    for node_id in &node_ids {
        writeln!(
            &mut output,
            "pub const {}: crate::ua::NodeId = crate::ua::NodeId::numeric(0, {});",
            node_id.name.to_uppercase(), node_id.id
        )
        .unwrap();
    }
    output.flush().unwrap();

    let out_path = out_dir.join("ns0_id.rs");
    let out_file = std::fs::File::create(&out_path).unwrap();
    let mut output = std::io::BufWriter::new(out_file);
    for node_id in &node_ids {
        writeln!(
            &mut output,
            "pub const {}: u32 = {};",
            node_id.name.to_uppercase(), node_id.id
        )
        .unwrap();
    }
    output.flush().unwrap();
}
