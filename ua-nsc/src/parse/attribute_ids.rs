use std::path;
use tokio::fs;
use tokio::io::{self, AsyncBufReadExt};

use super::Error;

#[derive(Debug, Clone)]
pub struct AttributeId {
    pub name: String,
    pub id: u32,
}

pub async fn parse_csv(csv_path: &path::Path) -> Result<Vec<AttributeId>, Error> {
    let mut attribute_ids = Vec::new();

    let reader = io::BufReader::new(fs::File::open(&csv_path).await?);
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        let sep = line
            .find(',')
            .ok_or_else(|| Error::Parse(format!("Missing first comma in line: {}", line)))?;

        let name = line[0..sep].to_string();
        let id = line[sep + 1..].parse()?;

        attribute_ids.push(AttributeId { name, id });
    }
    Ok(attribute_ids)
}
