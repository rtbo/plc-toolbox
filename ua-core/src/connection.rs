use crate::msg::binary::{self, DecodeBinary, EncodeBinary};
use crate::msg::uacp;


pub mod tcp;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("UACP error: {0}")]
    Uacp(#[from] uacp::Error),
    
    #[error("Decoding error: {0}")]
    Decoding(#[from] binary::DecodingError),
    
    #[error("Unsupported protocol: {0}")]
    UnsupportedProtocol(String),
}

#[allow(async_fn_in_trait)]
pub trait Connection: Sized {
    async fn connect(endpoint: &str) -> Result<Self, Error>;

    async fn send(&mut self, msg: impl EncodeBinary) -> Result<(), Error>;
    async fn recv<T: DecodeBinary>(&mut self) -> Result<T, Error>;
}
