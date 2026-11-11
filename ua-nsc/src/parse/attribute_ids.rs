use std::io::{self, BufRead};
use std::{fs, path};

use super::Error;

#[derive(Debug, Clone)]
pub struct AttributeId {
    pub name: String,
    pub id: u32,
}

pub fn parse_csv(csv_path: &path::Path) -> Result<Vec<AttributeId>, Error> {
    let mut attribute_ids = Vec::new();

    let reader = io::BufReader::new(fs::File::open(csv_path)?);
    for line in reader.lines() {
        let line = line?;
        let sep = line
            .find(',')
            .ok_or_else(|| Error::Parse(format!("Missing first comma in line: {}", line)))?;

        let name = line[0..sep].to_string();
        let id = line[sep + 1..].parse()?;

        attribute_ids.push(AttributeId { name, id });
    }
    Ok(attribute_ids)
}
