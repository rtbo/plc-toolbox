use std::collections::HashMap;
use std::sync::Arc;

use sc_bin::ScBinClient;
use tokio::net;
use ua_core::encoding::binary;
use ua_core::{SecurityPolicyNone, msg, ua};

mod sc_bin;
pub mod secure_channel;

mod smallmap;

/// Buffer configuration for the OPC UA client.
///
/// It is used for stack profiles based on UA Secure Conversation and Binary data encoding:
///  - UA-TCP UA-SC UA-Binary (URLs starting with `"opc.tcp://"`)
///  - WSS UA-SC UA-Binary (URLs starting with `"wss://"`)
///
/// The buffer configuration is negotiated between the client and the server during the handshake.
/// Therefore those values may be adjusted by the server during the connection process.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BufferConfig {
    /// The size of the send buffer.
    /// An initial size is set by the client, but it may be adjusted by the server during the handshake.
    /// The default is 64 KiB.
    pub send_size: usize,
    /// The size of the receive buffer.
    /// An initial size is set by the client, but it may be adjusted by the server during the handshake.
    /// The default is 64 KiB
    pub recv_size: usize,
    /// The maximum size of a request message.
    /// If `None`, there is no limit on the message size. (this is the default)
    ///
    /// This field may be set by the server during the handshake if the server imposes a limit on request message size.
    ///
    /// This field can also be set by the client to impose a lower memory limit than the server for request messages.
    /// In all cases, an error is returned when attempting to encode a request that exceeds the allowed size.
    pub max_request_size: Option<usize>,
    /// The maximum size of a response message.
    /// This will determine the maximum number of chunks a response can be split into.
    /// If `None`, there is no limit on the message size (this is the default).
    ///
    /// This field can be set by the client to impose to the server a memory limit for response messages.
    pub max_response_size: Option<usize>,
}

impl BufferConfig {
    fn chunk_count(msg_size: usize, chunk_size: usize) -> usize {
        (msg_size + chunk_size - 1) / chunk_size
    }

    pub fn max_send_chunks(&self) -> Option<usize> {
        self.max_request_size
            .map(|size| BufferConfig::chunk_count(size, self.send_size))
    }

    pub fn max_recv_chunks(&self) -> Option<usize> {
        self.max_response_size
            .map(|size| BufferConfig::chunk_count(size, self.recv_size))
    }
}

impl Default for BufferConfig {
    fn default() -> Self {
        Self {
            send_size: 65536,
            recv_size: 65536,
            max_request_size: None,
            max_response_size: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Config {
    /// The endpoint URL of the OPC UA server to connect to.
    /// The URL scheme will determine the stack profile to use for the connection
    pub endpoint: String,
    /// The buffer configuration for the client.
    pub buffer: BufferConfig,

    /// A map containing external nodeset URIs and the corresponding decoding function
    /// This can be built by the user by using the `ua_nsc` crate in a build script to generate
    /// the necessary decoding functions for external nodeset.
    ///
    /// By default, this map is empty and the client only decodes the NS0 nodeset.
    pub external_ns: HashMap<String, ua_core::encoding::binary::DecodingFn>,
}

/// An OPC/UA client.
///
/// This client supports connecting to an OPC UA server over TCP using the binary protocol.
///
/// This client manages interior mutability and is cheap to clone, so no need to put it into an `Arc` manually.
/// It can be sent across threads or tasks safely and be used concurrently "as-is".
#[derive(Debug, Clone)]
pub struct Client {
    inner: Arc<ClientImpl>,
}

impl Client {
    pub async fn connect(config: Config) -> Result<Self, ua::StatusCode> {
        let security_policy = SecurityPolicyNone;

        let client = if config.endpoint.starts_with("opc.tcp://") {
            let url = &config.endpoint["opc.tcp://".len()..];
            let stream = tokio::net::TcpStream::connect(url)
                .await
                .map_err(|_| ua::StatusCode::BadConnectionRejected)?;
            let (read_half, write_half) = stream.into_split();
            let sc_bin_cl =
                sc_bin::ScBinClient::connect(config, security_policy, write_half, read_half)
                    .await
                    .map_err(|_| ua::StatusCode::BadConnectionRejected)?;
            Self {
                inner: Arc::new(ClientImpl::TcpScBin(sc_bin_cl)),
            }
        } else {
            return Err(ua::StatusCode::BadInvalidArgument);
        };

        Ok(client)
    }

    pub fn config(&self) -> &Config {
        match &*self.inner {
            ClientImpl::TcpScBin(sc_bin_cl) => sc_bin_cl.config(),
        }
    }

    pub async fn send_request<R>(&self, request: R) -> Result<R::Response, ua::StatusCode>
    where
        R: msg::Request + binary::Encode + msg::BinaryEncodingId,
        R::Response: binary::Decode + msg::BinaryEncodingId,
    {
        match &*self.inner {
            ClientImpl::TcpScBin(sc_bin_cl) => sc_bin_cl.send_request(request).await,
        }
    }

    pub async fn close(&self) -> Result<(), ua::StatusCode> {
        match &*self.inner {
            ClientImpl::TcpScBin(sc_bin_cl) => {
                let _recv_half = sc_bin_cl.close().await?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
enum ClientImpl {
    TcpScBin(ScBinClient<SecurityPolicyNone, net::tcp::OwnedWriteHalf>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_chunks() {
        let tests = &[
            (1024, 256, 4),
            (1025, 256, 5),
            (1024, 512, 2),
            (1023, 512, 2),
            (1024, 1024, 1),
            (1024, 2048, 1),
        ];
        for &(msg_size, chunk_size, expected) in tests {
            assert_eq!(BufferConfig::chunk_count(msg_size, chunk_size), expected);
        }
    }
}
