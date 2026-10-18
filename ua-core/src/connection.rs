use crate::msg::binary;
use crate::msg::uacp::{self, Msg};

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

    #[error("Invalid or oversized message: {size} bytes (max {max})")]
    MessageSize { size: usize, max: usize },
}

#[allow(async_fn_in_trait)]
pub trait Connection: Sized {
    async fn connect(endpoint: &str) -> Result<Self, Error>;

    async fn send(&mut self, msg: Msg) -> Result<(), Error>;
    async fn recv(&mut self) -> Result<Msg, Error>;
}
