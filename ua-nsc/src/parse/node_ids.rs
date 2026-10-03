use std::{fmt, path};
use tokio::fs;
use tokio::io::{self, AsyncBufReadExt};

use super::Error;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NodeClass {
    Unspecified,
    Object,
    Variable,
    Method,
    ObjectType,
    VariableType,
    ReferenceType,
    DataType,
    View,
}

impl std::str::FromStr for NodeClass {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Unspecified" => Ok(NodeClass::Unspecified),
            "Object" => Ok(NodeClass::Object),
            "Variable" => Ok(NodeClass::Variable),
            "Method" => Ok(NodeClass::Method),
            "ObjectType" => Ok(NodeClass::ObjectType),
            "VariableType" => Ok(NodeClass::VariableType),
            "ReferenceType" => Ok(NodeClass::ReferenceType),
            "DataType" => Ok(NodeClass::DataType),
            "View" => Ok(NodeClass::View),
            _ => Err(Error::Parse(format!("Unknown node type: {}", s))),
        }
    }
}

impl fmt::Display for NodeClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NodeClass::Unspecified => write!(f, "Unspecified"),
            NodeClass::Object => write!(f, "Object"),
            NodeClass::Variable => write!(f, "Variable"),
            NodeClass::Method => write!(f, "Method"),
            NodeClass::ObjectType => write!(f, "ObjectType"),
            NodeClass::VariableType => write!(f, "VariableType"),
            NodeClass::ReferenceType => write!(f, "ReferenceType"),
            NodeClass::DataType => write!(f, "DataType"),
            NodeClass::View => write!(f, "View"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NodeId {
    pub name: String,
    pub numeric_id: u32,
    pub class: NodeClass,
}

impl NodeId {
    pub fn node_id_str(&self) -> String {
        format!("i={}", self.numeric_id)
    }
}

pub async fn parse_csv(csv_path: &path::Path) -> Result<Vec<NodeId>, Error> {
    let mut status_codes = Vec::new();

    let reader = io::BufReader::new(fs::File::open(&csv_path).await?);
    let mut lines = reader.lines();
    while let Some(line) = lines.next_line().await? {
        let sep1 = line
            .find(',')
            .ok_or_else(|| Error::Parse(format!("Missing first comma in line: {}", line)))?;
        let sep2 = line[sep1 + 1..]
            .find(',')
            .ok_or_else(|| Error::Parse(format!("Missing second comma in line: {}", line)))?;

        let name = line[0..sep1].to_string();
        let numeric_id = line[sep1 + 1..sep1 + 1 + sep2].parse()?;
        let class = line[sep1 + 1 + sep2 + 1..].parse()?;


        status_codes.push(NodeId {
            name,
            numeric_id,
            class,
        });
    }
    Ok(status_codes)
}
