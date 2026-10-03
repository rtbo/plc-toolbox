use std::fmt;
use tokio::io;

pub mod attribute_ids;
pub mod node_ids;
pub mod opc_bsd;
pub mod status_codes;

pub use attribute_ids::AttributeId;
pub use node_ids::{NodeClass, NodeId};
pub use status_codes::StatusCode;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Parse(String),
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(err: std::num::ParseIntError) -> Self {
        Error::Parse(err.to_string())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Parse(err) => write!(f, "Parse error: {}", err),
        }
    }
}

impl std::error::Error for Error {}
