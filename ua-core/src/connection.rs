use crate::msg::binary::EncodeBinary;
use crate::msg::uacp;

pub mod tcp;

pub enum Error {
    Io(std::io::Error),
    Uacp(uacp::Error),
    UnsupportedProtocol(String),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<uacp::Error> for Error {
    fn from(err: uacp::Error) -> Self {
        Error::Uacp(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {}", err),
            Error::Uacp(err) => write!(f, "UACP error: {}", err),
            Error::UnsupportedProtocol(proto) => write!(f, "Unsupported protocol: {}", proto),
        }
    }
}

#[allow(async_fn_in_trait)]
pub trait Connection: Sized {
    async fn connect(endpoint: &str) -> Result<Self, Error>;

    async fn send(&mut self, msg: impl EncodeBinary) -> Result<(), Error>;
}
