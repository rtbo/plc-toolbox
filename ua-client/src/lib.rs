use sc_bin::ScBinClient;
use tokio::net;
use ua_core::{encoding::binary, msg, ua};

mod sc_bin;
pub mod secure_channel;

mod smallmap;

/// Buffer configuration for the OPC UA client.
///
/// It is used for stack profiles based on UA Secure Conversation and Binary data encoding:
///  - UA-TCP UA-SC UA-Binary (URLs starting with `"opc.tcp://"`)
///  - WSS UA-SC UA-Binary (URLs starting with `"wss://"`)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferConfig {
    /// The size of the send buffer.
    pub send_size: usize,
    /// The size of the receive buffer.
    pub recv_size: usize,
    /// The maximum number of message chunks.
    pub max_msg_chunks: Option<usize>,
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            send_size: 65536,
            recv_size: 65536,
            max_msg_chunks: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Config {
    /// The endpoint URL of the OPC UA server to connect to.
    pub endpoint: String,
    /// The buffer configuration for the client.
    pub buffer: BufferConfig,
}

#[derive(Debug)]
pub struct Client {
    inner: ClientImpl,
}

impl Client {
    pub async fn connect(config: Config) -> Result<Self, ua::StatusCode> {
        if config.endpoint.starts_with("opc.tcp://") {
            let url = &config.endpoint["opc.tcp://".len()..];
            let stream = tokio::net::TcpStream::connect(url)
                .await
                .map_err(|_| ua::StatusCode::BadConnectionRejected)?;
            let (read_half, write_half) = stream.into_split();
            let sc_bin_cl = sc_bin::ScBinClient::connect(config, write_half, read_half)
                .await
                .map_err(|_| ua::StatusCode::BadConnectionRejected)?;
            Ok(Self {
                inner: ClientImpl::TcpScBin(sc_bin_cl),
            })
        } else {
            return Err(ua::StatusCode::BadInvalidArgument);
        }
    }

    pub async fn send_request<R>(&self, request: R) -> Result<R::Response, ua::StatusCode>
    where
        R: msg::Request + binary::Encode + msg::BinaryEncodingId,
        R::Response: binary::Decode + msg::BinaryEncodingId,
    {
        match &self.inner {
            ClientImpl::TcpScBin(sc_bin_cl) => sc_bin_cl.send_request(request).await,
        }
    }

    pub async fn close(&self) -> Result<(), ua::StatusCode> {
        match &self.inner {
            ClientImpl::TcpScBin(sc_bin_cl) => {
                let _recv_half = sc_bin_cl.close().await?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
enum ClientImpl {
    TcpScBin(ScBinClient<net::tcp::OwnedWriteHalf>),
}
